use cursive::Cursive;

pub struct AppState {
    pub loading: bool,
    pub streaming: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            loading: false,
            streaming: false,
        }
    }

    pub fn init(s: &mut Cursive) {
        let app_state = Self::new();
        s.set_user_data(app_state);
    }
}

pub fn set_is_loading(s: &mut Cursive) {
    let user_data = s.user_data::<AppState>();
    if let Some(user_data) = user_data {
        user_data.loading = true;
        user_data.streaming = false;
    }
}

pub fn set_is_streaming(s: &mut Cursive) {
    let user_data = s.user_data::<AppState>();
    if let Some(user_data) = user_data {
        user_data.loading = false;
        user_data.streaming = true;
    }
}

pub fn set_ready(s: &mut Cursive) {
    let user_data = s.user_data::<AppState>();
    if let Some(user_data) = user_data {
        user_data.loading = false;
        user_data.streaming = false;
    }
}

pub fn get_is_loading(s: &mut Cursive) -> bool {
    s.user_data::<AppState>()
        .map(|x| x.loading)
        .unwrap_or(false)
}

pub fn get_is_streaming(s: &mut Cursive) -> bool {
    s.user_data::<AppState>()
        .map(|x| x.loading)
        .unwrap_or(false)
}
