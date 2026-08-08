//! Pure encoding for asynchronous host command results and timer callbacks.

use std::collections::BTreeMap;

use crate::agent_refresh::RequestId;
use crate::app::Message;

const RESULT_KIND: &str = "session_picker_result";
const RESULT_SESSION: &str = "session_picker_session";
const RESULT_REQUEST_ID: &str = "session_picker_request_id";

#[derive(Debug, Clone, Copy)]
pub enum ResultKind {
    Agents { request_id: RequestId },
    Directory,
    Git,
    WorktreeValidation,
    WorktreeCreation,
}

impl ResultKind {
    fn tag(self) -> &'static str {
        match self {
            Self::Agents { .. } => "agents",
            Self::Directory => "directory",
            Self::Git => "git",
            Self::WorktreeValidation => "worktree_validation",
            Self::WorktreeCreation => "worktree_creation",
        }
    }
}

pub fn result_context(kind: ResultKind, session: Option<String>) -> BTreeMap<String, String> {
    let mut context = BTreeMap::from([(RESULT_KIND.into(), kind.tag().into())]);
    if let ResultKind::Agents { request_id } = kind {
        context.insert(RESULT_REQUEST_ID.into(), request_id.0.to_string());
    }
    if let Some(session) = session {
        context.insert(RESULT_SESSION.into(), session);
    }
    context
}

pub fn decode_result(
    exit_code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
    context: &BTreeMap<String, String>,
) -> Option<Message> {
    match context.get(RESULT_KIND).map(String::as_str)? {
        "agents" => {
            let request_id = RequestId(context.get(RESULT_REQUEST_ID)?.parse().ok()?);
            let result = if exit_code == Some(0) {
                crate::agents::parse_list(stdout).map_err(|_| ())
            } else {
                Err(())
            };
            Some(Message::AgentsFetchFinished { request_id, result })
        }
        "directory" => Some(Message::DirectoryCandidatesLoaded(
            crate::create::discovery::parse_zoxide_result(stdout),
        )),
        "git" => Some(Message::GitLoaded {
            session_name: context.get(RESULT_SESSION)?.clone(),
            info: crate::git_info::GitInfo::parse(stdout),
        }),
        "worktree_validation" => Some(Message::WorktreeValidationFinished {
            result: command_result(exit_code, stderr),
        }),
        "worktree_creation" => Some(Message::WorktreeCreationFinished {
            result: command_result(exit_code, stderr),
        }),
        _ => None,
    }
}

fn command_result(exit_code: Option<i32>, stderr: &[u8]) -> Result<(), String> {
    if exit_code == Some(0) {
        return Ok(());
    }
    let error = String::from_utf8_lossy(stderr).trim().to_string();
    Err(if error.is_empty() {
        "command failed".into()
    } else {
        error
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tagged_result_decodes_to_exactly_one_message() {
        let context = result_context(ResultKind::Directory, None);
        let message = decode_result(Some(0), b"/tmp/a\n/tmp/b\n", b"", &context);
        assert!(
            matches!(message, Some(Message::DirectoryCandidatesLoaded(paths)) if paths.len() == 2)
        );
    }

    #[test]
    fn an_unknown_result_is_not_consumed() {
        assert!(decode_result(Some(0), b"anything", b"", &BTreeMap::new()).is_none());
    }

    #[test]
    fn agent_failures_preserve_request_identity() {
        let context = result_context(
            ResultKind::Agents {
                request_id: RequestId(42),
            },
            None,
        );
        let message = decode_result(Some(1), b"", b"failed", &context);
        assert!(matches!(
            message,
            Some(Message::AgentsFetchFinished {
                request_id: RequestId(42),
                result: Err(())
            })
        ));
    }
}
