pub enum Environment {
    Dev,
    Prod
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        "Dev"
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "dev" => Ok(Self::Dev),
            "prod" | "production" => Ok(Self::Prod),
            _ => Err(format!("{} is not a supported environment", value))
        }
    }
}
