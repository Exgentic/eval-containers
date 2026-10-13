//! The OSWorld desktop's agent door (`core/osworld-desktop/agent_face.py`) is all
//! that stands between a shell-equipped agent and a server that runs commands
//! in the session. `/act` takes pyautogui source, which is Python source, so the
//! door admits a line only if it is `pyautogui.<allowed>(…)` or `time.sleep(…)`
//! on literal arguments. This runs the shipped check on both sides of that line.

use std::process::Command;
use test_support::repo_root;

/// Whether the door would run `action`.
fn admits(action: &str) -> bool {
    // 3 is the door refusing; any other failure is the check itself not running.
    let check = "import sys, agent_face\n\
                 try:\n    agent_face.checked(sys.argv[1])\n\
                 except (ValueError, SyntaxError):\n    sys.exit(3)";
    let status = Command::new("python3")
        .args(["-B", "-c", check, action])
        .current_dir(repo_root().join("containers/core/osworld-desktop"))
        .status()
        .expect("run python3");
    match status.code() {
        Some(0) => true,
        Some(3) => false,
        code => panic!("the door's check did not run: {code:?}"),
    }
}

#[test]
fn pyautogui_on_literals_is_admitted() {
    for action in [
        "pyautogui.click(x=100, y=200)",
        "pyautogui.hotkey('ctrl', 's')",
        // A list of keys is how `press` and `write` take a sequence.
        "pyautogui.press(['tab', 'tab', 'enter'])",
        "pyautogui.typewrite('=SUM(A1:A9)', interval=0.05)\ntime.sleep(0.5)",
    ] {
        assert!(admits(action), "refused a plain pyautogui action: {action}");
    }
}

#[test]
fn anything_richer_is_refused() {
    for action in [
        "__import__('os').system('id')",
        "open('/home/user/answer.xlsx', 'w')",
        "import os",
        "os.system('id')",
        "x = pyautogui.position()",
        "pyautogui.__class__",
        "pyautogui.run('id')",
        "pyautogui.screenshot().save('/home/user/answer.png')",
        "pyautogui.click(x=len(open('/etc/passwd').read()))",
        "pyautogui.write([open(p).read() for p in ['/etc/passwd']])",
        "pyautogui.write(f'{open}')",
        "pyautogui.click(*[1, 2])",
        "pyautogui.click(**{'x': 1})",
        "pyautogui.click(1, 2)\nimport os",
    ] {
        assert!(
            !admits(action),
            "admitted more than a plain pyautogui call: {action}"
        );
    }
}
