# oloko64-dev email sender

This is a simple email sender written in Rust. Its used in [my website](https://www.oloko64.dev/) to send notifications to me via email and Telegram.

It has the option to be deployed as a normal web server or as a lambda function in AWS without any code changes.

### AWS Build

To build for AWS Lambda functions you need to target `x86_64-unknown-linux-musl`, using [cross](https://github.com/cross-rs/cross) you can run the following commands:

```rs
cross build --release --target x86_64-unknown-linux-musl

cp target/x86_64-unknown-linux-musl/release/your_lambda_app_name ./bootstrap

zip lambda.zip bootstrap
```

[AWS Docs](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/lambda.html)
[AWS Example](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples/http-axum)

### CI Deploy (GitHub Actions)

Every push to `main` runs the tests, builds the Lambda with `cross` and updates the `email-sender-oloko64-dev` function in `us-east-1` (see `.github/workflows/deploy-lambda.yml`). It can also be triggered manually from the Actions tab.

Authentication uses GitHub OIDC (no AWS keys stored in GitHub). One-time setup:

1. Create the GitHub OIDC provider in IAM (skip if it already exists):

```sh
aws iam create-open-id-connect-provider \
  --url https://token.actions.githubusercontent.com \
  --client-id-list sts.amazonaws.com
```

2. Create a role (e.g. `github-deploy-email-sender`) with this trust policy (replace `<ACCOUNT_ID>`):

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": { "Federated": "arn:aws:iam::<ACCOUNT_ID>:oidc-provider/token.actions.githubusercontent.com" },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
          "token.actions.githubusercontent.com:sub": "repo:oloko64/oloko64-dev-email-sender-rust:ref:refs/heads/main"
        }
      }
    }
  ]
}
```

3. Attach this inline permission policy to the role:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["lambda:UpdateFunctionCode", "lambda:GetFunction", "lambda:GetFunctionConfiguration"],
      "Resource": "arn:aws:lambda:us-east-1:<ACCOUNT_ID>:function:email-sender-oloko64-dev"
    }
  ]
}
```

4. In the GitHub repo, go to **Settings → Secrets and variables → Actions → Variables** and add `AWS_ROLE_ARN` with the role ARN (`arn:aws:iam::<ACCOUNT_ID>:role/github-deploy-email-sender`).
