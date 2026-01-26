use tracing::field::Field;
use tracing_subscriber::field::Visit;

#[derive(Debug)]
pub(super) struct ClickhouseVisitor {
    pub(super) extras: Vec<Extra>,
    pub(super) file: Option<String>,
    pub(super) line: Option<u64>,
    pub(super) module_path: Option<String>,
    pub(super) target: Option<String>,
    pub(super) message: Option<String>,
    pub(super) causes: Option<String>
}

#[derive(Debug, serde::Serialize, Clone)]
pub(super) struct Extra(pub(super) &'static str, pub(super) String);

impl ClickhouseVisitor {
    pub(super) fn new() -> Self {
        Self {
            extras: Vec::new(),
            file: None,
            line: None,
            module_path: None,
            target: None,
            message: None,
            causes: None,
        }
    }
}

impl ClickhouseVisitor {
    fn record_extra(&mut self, f: &Field, v: &str) {
        self.extras
            .push(Extra(f.name(), v.into()));
    }
}

impl Visit for ClickhouseVisitor {
    fn record_u64(&mut self, f: &Field, v: u64) {
        match f.name() {
            "log.line" => self.line = Some(v),
            _ => self.record_extra(f, format!("{}", v).as_str()),
        }
    }

    fn record_str(&mut self, f: &Field, v: &str) {
        match f.name() {
            "log.file" => self.file = Some(v.into()),
            "log.module_path" => self.module_path = Some(v.into()),
            "module_path" => self.module_path = Some(v.into()),
            "causes" => self.causes = Some(v.into()),
            "target" => self.target = Some(v.into()),
            _ => self.record_extra(f, &v),
        }
    }

    fn record_debug(&mut self, f: &Field, v: &dyn std::fmt::Debug) {
        match f.name() {
            "message" => self.message = Some(format!("{:?}", v).into()),
            _ => self.record_extra(f, format!("{:?}", v).as_str()),
        }
    }
}
