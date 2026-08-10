use crate::create::CreateFlow;
use crate::ui::screens::list::UiState as ListUiState;

#[derive(Default)]
pub(crate) enum Screen {
    #[default]
    List,
    Rename {
        original: String,
        draft: String,
    },
    Create(Box<CreateFlow>),
}

#[derive(Default)]
pub(crate) struct UiState {
    pub(crate) screen: Screen,
    pub(crate) list: ListUiState,
}
