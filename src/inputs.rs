use crate::input::read_line;
use anyhow::Result;

enum Inputs {
    CommitCommand,
    BranchName,
    RemoteName,
    RemoteUrl,
}

impl Inputs {
    pub fn call(&self) -> Result<String> {
        let prompt = self.prompt();
        let input = read_line(&prompt)?;
        Ok(input)
    }

    pub fn prompt(&self) -> String {
        match self {
            Self::CommitCommand => "Please input commit message:".to_string(),
            Self::BranchName => "Please input branch name:".to_string(),
            Self::RemoteName => "Please input remote name:".to_string(),
            Self::RemoteUrl => "Please input remote url:".to_string(),
        }
    }
}
