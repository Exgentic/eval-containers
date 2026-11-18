"""Write the AssertLLM2-verify task store: one line per (design, variant).

usage: build-tasks.py <upstream tree> <designs.txt> <out dir> [<revision>]

Writes <out>/all.jsonl, <out>/report.json (what was kept and why the rest was
not), and <out>/assets/<design>/ (the figures its spec shows, stored once).

Per kept design (designs.txt): the golden RTL, five of its twenty individual
mutants picked with a fixed seed, and its merged variant. Labels come from the
origin: golden -> correct, both mutant kinds -> incorrect.

Every variant of a design must look alike apart from its edits. Upstream wrote
each mutated file with PyVerilog's code generator (no comments, its own layout)
and left the rest as authored, so a file in that layout would itself say
"mutated". So for each design, every file that any of its selected variants
changes is put through that same emitter in every variant, the golden included,
repeated until its output stops changing (one pass is not stable), with the
golden file's leading comment block (its license notice) on top; every
other file is the golden's, with line endings and trailing blanks normalized.
The emitter is upstream's own: PyVerilog 1.2.1 with upstream's patch, then
upstream's sanitizer, taken from its source tree. Two of upstream's own steps
are left out or undone, because they break the RTL for every tool here:
upstream's translate_off/on restorer puts the pragmas back by guesswork and can
wrap whole modules, so the regenerated files carry none (as their comments go,
so do the pragmas); and the patched generator writes an empty statement as
`@(*);`, which is put back as `;`.

Preprocessing fixes every macro at the value upstream builds with, so a spec
that names one of them gets a note saying which configuration the RTL is in.
"""

import ast
import json
import random
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

INDIVIDUAL_PER_DESIGN = 5
RTL = (".v", ".sv", ".vh", ".h", ".svh")


def upstream_functions(path, names):
    """The named top-level functions of an upstream module, and what they use."""
    tree = ast.parse(Path(path).read_text())
    nodes = {}
    for n in tree.body:
        if isinstance(n, ast.FunctionDef):
            nodes[n.name] = n
        elif isinstance(n, ast.Assign):
            nodes.update({t.id: n for t in n.targets if isinstance(t, ast.Name)})
    want, todo = set(), list(names)
    while todo:
        k = todo.pop()
        if k in nodes and k not in want:
            want.add(k)
            todo += [x.id for x in ast.walk(nodes[k]) if isinstance(x, ast.Name)]
    keep = {id(nodes[k]) for k in want}
    ns = {"re": re}
    body = [n for n in tree.body if id(n) in keep]
    exec(compile(ast.Module(body=body, type_ignores=[]), str(path), "exec"), ns)
    return [ns[n] for n in names]


def text_normal(s):
    lines = s.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    return "\n".join(line.rstrip() for line in lines).strip("\n") + "\n"


COND = re.compile(r"`(?:ifdef|ifndef|elsif)\s+(\w+)")


def configuration(root, filelist, rels, spec):
    """The note a spec gets on the one configuration its RTL is supplied in.

    The emitter preprocesses what it regenerates, so every macro the RTL tests
    is fixed at what upstream builds it with: nothing defined on the command
    line (upstream's configs name no defines). A spec that names such a macro
    describes other configurations too, and a golden that cannot be put in one
    of them reads as wrong, so the spec says which one this is. Icarus Verilog,
    the emitter's own preprocessor, says what each macro ends up as.
    """
    tested = {
        m for r in rels for m in COND.findall((root / r).read_text(errors="replace"))
    }
    names = sorted(m for m in tested if re.search(rf"\b{re.escape(m)}\b", spec))
    if not names:
        return ""
    with tempfile.TemporaryDirectory() as t:
        probe, out = Path(t) / "probe.v", Path(t) / "out.v"
        probe.write_text(
            "".join(
                f"`ifdef {m}\n@@ {m} = `{m}\n`else\n@@ {m}\n`endif\n" for m in names
            )
        )
        subprocess.run(
            ["iverilog", "-E", "-o", str(out), "-I", str(root / "include")]
            + [str(root / f) for f in filelist]
            + [str(probe)],
            check=True,
            capture_output=True,
        )
        seen = dict(
            (m.group(1), m.group(2))
            for m in re.finditer(r"^\s*@@ (\w+)(?: = (.*))?$", out.read_text(), re.M)
        )
    lines = []
    for m in names:
        v = seen[m]
        if v is None:
            lines.append(f"- `{m}` is not defined")
        elif v.strip():
            lines.append(f"- `{m}` is defined as `{v.strip()}`")
        else:
            lines.append(f"- `{m}` is defined")
    return (
        "\n\n## Configuration of the supplied RTL\n\n"
        "The RTL is supplied in one configuration, the one it is built with when "
        "no macro is defined on the command line:\n\n"
        + "\n".join(lines)
        + "\n\nJudge it in this configuration; other configurations are not part "
        "of this task.\n"
    )


def leading_comments(text):
    out, in_block = [], False
    for line in text.split("\n"):
        s = line.strip()
        if in_block:
            in_block = "*/" not in s
        elif s.startswith("/*"):
            in_block = "*/" not in s
        elif s and not s.startswith("//"):
            break
        out.append(line)
    return "\n".join(out).strip("\n")


class Emitter:
    def __init__(self, upstream):
        (self.sanitize,) = upstream_functions(
            upstream / "mutation/main.py", ["sanitize_mutated_rtl_text"]
        )
        from pyverilog.ast_code_generator.codegen import ASTCodeGenerator
        from pyverilog.vparser.parser import parse

        self.parse, self.gen = parse, ASTCodeGenerator()

    def __call__(self, tree, rel, golden):
        """`rel` of the materialized variant `tree`, as upstream wrote its mutants.

        Re-emitted until it stops changing: one pass writes `reg a, b;` as
        `reg a;reg b;` on one line and the next splits it, so a golden taken
        from source would differ in layout from a mutant, which upstream already
        emitted once. Writes the result back into `tree`.
        """
        prev = None
        for _ in range(4):
            with tempfile.TemporaryDirectory() as out:
                parsed, _ = self.parse(
                    [str(tree / rel)],
                    preprocess_include=[str(tree / "include")],
                    outputdir=out,
                    debug=False,
                )
            code, _ = self.sanitize(self.gen.visit(parsed))
            code = text_normal(code.replace("@(*);", ";"))
            if code == prev:
                break
            (tree / rel).write_text(code)
            prev = code
        else:
            raise ValueError("emitter output does not settle")
        notice = leading_comments(golden)
        return f"{notice}\n\n{code}" if notice else code


def main():
    upstream, designs_txt, out = map(Path, sys.argv[1:4])
    revision = sys.argv[4] if len(sys.argv) > 4 else None
    configs = json.loads(
        (upstream / "AssertLLM2/configs/assertllm2_design_configs.json").read_text()
    )
    keep = [
        line.split()[0]
        for line in designs_txt.read_text().splitlines()
        if line.strip() and not line.startswith("#")
    ]
    emit = Emitter(upstream)
    rows, report = [], {"revision": revision, "designs": {}, "excluded": []}

    for key in keep:
        cfg = configs[key]
        name = key.rsplit("/", 1)[1]
        root = upstream / Path(cfg["spec_file"][0].replace("../", "")).parent
        filelist = [
            str(Path(f.replace("../", "")).relative_to(root.relative_to(upstream)))
            for f in cfg["rtl"]["filelist"]
        ]
        # The filelist plus every RTL header its include directory holds.
        rels = list(filelist) + sorted(
            str(p.relative_to(root))
            for p in (root / "include").glob("*")
            if p.suffix in RTL and str(p.relative_to(root)) not in filelist
        )
        golden = {r: text_normal((root / r).read_text(errors="replace")) for r in rels}
        by_name = {Path(r).name: r for r in rels}
        assert len(by_name) == len(rels), f"{name}: RTL basenames are not unique"

        def overlay(d):
            return {
                by_name[p.name]: text_normal(p.read_text(errors="replace"))
                for p in sorted(d.glob("*"))
                if p.name in by_name
            }

        picks = sorted(
            random.Random(f"assertllm2-verify:{name}").sample(
                range(20), INDIVIDUAL_PER_DESIGN
            )
        )
        variants = [("golden", "golden", {})]
        variants += [
            (
                f"M_{m:04d}",
                "individual_mutant",
                overlay(root / f"mutations/mutants/M_{m:04d}"),
            )
            for m in picks
        ]
        variants += [
            ("merged", "merged", overlay(root / "buggy_artifacts/merged_buggy_rtl"))
        ]
        changed = {
            v: {r for r, s in over.items() if s != golden[r]} for v, _, over in variants
        }
        regenerated = set().union(*changed.values())

        def tree_of(over):
            t = Path(tempfile.mkdtemp())
            for r in rels:
                (t / r).parent.mkdir(parents=True, exist_ok=True)
                (t / r).write_text(over.get(r, golden[r]))
            return t

        built = {}
        for v, kind, over in variants:
            tree = tree_of(over)
            try:
                files = {
                    f"rtl/{r}": emit(tree, r, golden[r])
                    if r in regenerated
                    else over.get(r, golden[r])
                    for r in rels
                }
            except Exception as e:  # PyVerilog cannot parse it: not a usable task
                report["excluded"].append(
                    {
                        "id": f"{name}/{v}",
                        "reason": f"emitter: {type(e).__name__}: {str(e)[:200]}",
                    }
                )
                continue
            finally:
                shutil.rmtree(tree)
            built[v] = (kind, files)

        if "golden" not in built:
            report["excluded"].append(
                {
                    "id": f"{name}/*",
                    "reason": "golden RTL does not go through the emitter",
                }
            )
            continue
        for v in list(built):
            if v != "golden" and built[v][1] == built["golden"][1]:
                report["excluded"].append(
                    {
                        "id": f"{name}/{v}",
                        "reason": "identical to the golden after normalization",
                    }
                )
                del built[v]

        spec = (root / "spec.md").read_text()
        note = configuration(root, filelist, rels, spec)
        if note:
            spec = spec.rstrip("\n") + note
        if (root / "figures").is_dir():
            shutil.copytree(root / "figures", out / "assets" / name / "figures")
        # What a tool needs to build it, in upstream's compile order.
        files_f = "+incdir+include\n" + "".join(f"{r}\n" for r in filelist)
        for v, (kind, files) in built.items():
            files["rtl/files.f"] = files_f
            rows.append(
                {
                    "id": f"{name}/{v}",
                    "design": name,
                    "label": "correct" if kind == "golden" else "incorrect",
                    "variant_kind": kind,
                    "top_module": cfg["rtl"]["top_module"],
                    "spec": spec,
                    # A string: eval-materialize-task writes each field as text.
                    "files": json.dumps(files),
                }
            )
        report["designs"][name] = {
            "key": key,
            "upstream": str(root.relative_to(upstream)),
            "sampled_mutants": [f"M_{m:04d}" for m in picks],
            "regenerated_files": sorted(regenerated),
            "tasks": [f"{name}/{v}" for v in built],
        }

    (out / "all.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
    (out / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(
        f"wrote {len(rows)} tasks from {len(report['designs'])} designs; {len(report['excluded'])} exclusions"
    )


if __name__ == "__main__":
    main()
