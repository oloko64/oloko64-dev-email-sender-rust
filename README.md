# oloko64-dev email sender

This is a simple email sender written in Rust. Its used in [my website](https://www.oloko64.dev/) to send emails to me.

It has the option to be deployed as a normal web server or as a lambda function in AWS without any code changes.

### AWS Build

To build for AWS Lambda functions you need to target `x86_64-unknown-linux-musl`, using [cross](https://github.com/cross-rs/cross) you can run the following commands:

```rs
cross build --release --target x86_64-unknown-linux-musl

cp target/x86_64-unknown-linux-musl/release/your_lambda_app_name ./bootstrap

zip lambda.zip bootstrap
```

[AWS Docs](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/lambda.html)
