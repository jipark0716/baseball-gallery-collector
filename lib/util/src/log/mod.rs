#[macro_export]
macro_rules! report {
    ($err:expr, $msg:expr) => {{
        use anyhow::Context;

        let causes: String = $err
            .chain()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        tracing::error!(
            causes = &causes,
            "{} err: {}", $msg, $err
        );
    }};
}
