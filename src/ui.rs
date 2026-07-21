use crate::database::Database;
use crate::models::Task as AppTask;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

pub fn launch_ui() -> iced::Result {
    iced::run(update, view)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Tasks,
    Analytics,
}

pub struct ProductivityApp {
    tasks: Vec<AppTask>,
    db: Database,
    draft_task_title: String,
    active_tab: Tab,
}

impl Default for ProductivityApp {
    fn default() -> Self {
        let db = Database::new("tasks.db");
        let tasks = db.fetch_tasks().unwrap_or_default();
        Self {
            tasks,
            db,
            draft_task_title: String::new(),
            active_tab: Tab::Tasks,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTask(i64, bool),
    DraftTitleChanged(String),
    SubmitTask,
    SwitchTab(Tab),
}

pub fn update(state: &mut ProductivityApp, message: Message) {
    match message {
        Message::SwitchTab(tab) => {
            state.active_tab = tab;
        }
        Message::ToggleTask(id, _) => {
            let _ = state.db.mark_task_completed(id);
            state.tasks = state.db.fetch_tasks().unwrap_or_default();
        }
        Message::DraftTitleChanged(title) => {
            state.draft_task_title = title;
        }
        Message::SubmitTask => {
            if !state.draft_task_title.is_empty() {
                let new_task = AppTask {
                    id: 0,
                    title: state.draft_task_title.clone(),
                    description: None,
                    priority: crate::models::Priority::Medium,
                    duration_minutes: None,
                    due_date: None,
                    is_completed: false,
                    is_notified: false,
                    recurring_rule: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                let _ = state.db.insert_task(&new_task);
                state.draft_task_title.clear();
                state.tasks = state.db.fetch_tasks().unwrap_or_default();
            }
        }
    }
}

pub fn view(state: &ProductivityApp) -> Element<'_, Message> {
    let header = row![
        button(text("Tasks").size(20))
            .padding(10)
            .on_press(Message::SwitchTab(Tab::Tasks)),
        button(text("Analytics").size(20))
            .padding(10)
            .on_press(Message::SwitchTab(Tab::Analytics)),
    ]
    .spacing(20)
    .align_y(iced::Alignment::Center);

    let active_content: Element<'_, Message> = match state.active_tab {
        Tab::Tasks => {
            let title = text("My Tasks").size(40);
            let input = text_input("What needs to be done?", &state.draft_task_title)
                .on_input(Message::DraftTitleChanged)
                .on_submit(Message::SubmitTask)
                .padding(15)
                .size(20);
            let submit_btn = button(text("Add Task").size(20))
                .padding(15)
                .on_press(Message::SubmitTask);
            let form_row = row![input, submit_btn]
                .spacing(15)
                .align_y(iced::Alignment::Center);

            let tasks_column = state.tasks.iter().filter(|t| !t.is_completed).fold(
                column![].spacing(15),
                |col, task| {
                    let check = row![
                        checkbox(task.is_completed)
                            .on_toggle(move |b| Message::ToggleTask(task.id, b))
                            .size(24),
                        text(&task.title).size(22)
                    ]
                    .spacing(15)
                    .align_y(iced::Alignment::Center);
                    col.push(check)
                },
            );
            column![title, form_row, scrollable(tasks_column)]
                .spacing(30)
                .into()
        }
        Tab::Analytics => {
            let title = text("Productivity Analytics").size(40);
            let total = state.tasks.len();
            let completed = state.tasks.iter().filter(|t| t.is_completed).count();
            let pending = total - completed;

            column![
                title,
                text(format!("Total Tasks Created: {}", total)).size(24),
                text(format!("Tasks Completed: {}", completed)).size(24),
                text(format!("Tasks Pending: {}", pending)).size(24),
            ]
            .spacing(20)
            .into()
        }
    };

    let layout = column![header, active_content].spacing(40).padding(40);

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
