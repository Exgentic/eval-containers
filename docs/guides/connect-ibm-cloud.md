# Connect to the IBM Cloud OpenShift cluster

*Guide · for operators · prerequisite for [Deploy on OpenShift](deploy-on-openshift.md).*

This gets you authenticated to the ETE VPC OpenShift cluster so `oc` (and the
deploy guide) can reach it. The SSO and `oc login` steps open a browser, so run
them yourself in a terminal.

## Mental model

Your IBM Cloud identity spans several sub-accounts. The cluster lives in the
**ETE VPC** account. One `ibmcloud login` bootstraps the session; you switch
accounts with `ibmcloud target -c <account-id>`.

`ibmcloud login` and `oc login` are **independent sessions**: switching IBM
Cloud accounts does not re-auth `oc`, and the `oc` session expires on its own.

| Account | ID | Used for |
|---|---|---|
| ETE VPC | `3231c832d5d14d9e9787b70bdaa1469f` | the OpenShift cluster |
| RIS3 | `045077ab9f1a4dcfafa2c58389d3d639` | COS storage (separate workflows) |

Cluster: `etevpc-int-shared-us-east` (region `us-east`), namespace `exgentic-ns`.

## 1. Install the tooling

You need `oc`, the `ibmcloud` CLI, and its OpenShift plugin:

```bash
# ibmcloud CLI
curl -fsSL https://clis.cloud.ibm.com/install/osx | sh   # macOS; see clis.cloud.ibm.com for linux/win
ibmcloud plugin install container-service
```

Verify:

```bash
ibmcloud --version
oc version --client
```

## 2. Log in to IBM Cloud (SSO)

```bash
ibmcloud login --sso
```

It prints a URL — open it, log in with your IBM ID, paste the one-time
passcode back. When prompted, pick region `us-east` and the **ETE VPC** account.

Confirm:

```bash
ibmcloud target
```

If you logged into a different account, switch:

```bash
ibmcloud target -c 3231c832d5d14d9e9787b70bdaa1469f
```

## 3. Write the cluster kubeconfig

```bash
ibmcloud oc cluster config -c etevpc-int-shared-us-east
```

## 4. Authenticate `oc` (browser)

```bash
oc login --web
```

After it succeeds, select the namespace:

```bash
oc whoami
oc project exgentic-ns
```

## You're connected

`oc` now targets the cluster and namespace. Continue with
[Deploy on OpenShift](deploy-on-openshift.md).

If `oc` later returns an auth error, the session expired — re-run
`oc login --web`. Switching IBM Cloud accounts with `ibmcloud target` does not
affect it.
