use crate::database::Database;
use crate::models::{Priority, Task as AppTask};
use chrono::{Duration as ChronoDuration, Local, NaiveDateTime, Utc};
use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Theme};

pub fn launch_ui() -> iced::Result {
    iced::application(ProductivityApp::default, update, view)
        .title("TickTick Clone")
        .theme(Theme::Dark)
        .run()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Tasks,
    Calendar,
    Analytics,
}

pub struct ProductivityApp {
    tasks: Vec<AppTask>,
    db: Database,
    draft_task_title: String,
    draft_task_desc: String,
    draft_task_priority: Priority,
    draft_duration_str: String,
    draft_due_date: Option<NaiveDateTime>,
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
            draft_task_desc: String::new(),
            draft_task_priority: Priority::Medium,
            draft_duration_str: String::new(),
            draft_due_date: None,
            active_tab: Tab::Tasks,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTask(i64),
    DraftTitleChanged(String),
    DraftDescChanged(String),
    SetDraftPriority(Priority),
    DraftDurationChanged(String),
    SetDraftDate(Option<NaiveDateTime>),
    SubmitTask,
    SwitchTab(Tab),
}

pub fn update(state: &mut ProductivityApp, message: Message) {
    match message {
        Message::SwitchTab(tab) => state.active_tab = tab,
        Message::ToggleTask(id) => {
            let _ = state.db.mark_task_completed(id);
            state.tasks = state.db.fetch_tasks().unwrap_or_default();
        }
        Message::DraftTitleChanged(title) => state.draft_task_title = title,
        Message::DraftDescChanged(desc) => state.draft_task_desc = desc,
        Message::SetDraftPriority(prio) => state.draft_task_priority = prio,
        Message::DraftDurationChanged(dur) => state.draft_duration_str = dur,
        Message::SetDraftDate(date) => state.draft_due_date = date,
        Message::SubmitTask => {
            if !state.draft_task_title.is_empty() {
                let due_date = state.draft_due_date.map(|d| d.and_utc());
                let desc = if state.draft_task_desc.is_empty() {
                    None
                } else {
                    Some(state.draft_task_desc.clone())
                };
                let duration_minutes = state.draft_duration_str.parse::<i32>().ok();

                let new_task = AppTask {
                    id: 0,
                    title: state.draft_task_title.clone(),
                    description: desc,
                    priority: state.draft_task_priority.clone(),
                    duration_minutes,
                    due_date,
                    is_completed: false,
                    is_notified: false,
                    recurring_rule: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                let _ = state.db.insert_task(&new_task);

                state.draft_task_title.clear();
                state.draft_task_desc.clear();
                state.draft_duration_str.clear();
                state.draft_task_priority = Priority::Medium;
                state.draft_due_date = None;

                state.tasks = state.db.fetch_tasks().unwrap_or_default();
            }
        }
    }
}

pub fn view(state: &ProductivityApp) -> Element<'_, Message> {
    let sidebar = container(
        column![
            text("TickTick Clone").size(28),
            Space::new().height(Length::Fixed(40.0)),
            button(text("📝    Tasks").size(18))
                .padding(15)
                .width(Length::Fill)
                .on_press(Message::SwitchTab(Tab::Tasks)),
            button(text("📅    Calendar").size(18))
                .padding(15)
                .width(Length::Fill)
                .on_press(Message::SwitchTab(Tab::Calendar)),
            button(text("📊    Analytics").size(18))
                .padding(15)
                .width(Length::Fill)
                .on_press(Message::SwitchTab(Tab::Analytics)),
        ]
        .spacing(15),
    )
    .padding(30)
    .width(Length::Fixed(260.0))
    .height(Length::Fill);

    let active_content: Element<'_, Message> = match state.active_tab {
        Tab::Tasks => {
            let title = text("Inbox").size(36);

            let input_title = text_input("Add a task...", &state.draft_task_title)
                .on_input(Message::DraftTitleChanged)
                .padding(15)
                .size(18);
            let input_desc = text_input("Description...", &state.draft_task_desc)
                .on_input(Message::DraftDescChanged)
                .padding(15)
                .size(18);
            let input_duration = text_input("Duration (mins)...", &state.draft_duration_str)
                .on_input(Message::DraftDurationChanged)
                .padding(15)
                .size(18);

            let now = Local::now().naive_local();
            let today_6pm = now.date().and_hms_opt(18, 0, 0).unwrap();
            let tomorrow = today_6pm + ChronoDuration::days(1);
            let next_week = today_6pm + ChronoDuration::days(7);

            let date_chips = row![
                text("Alarm:").size(16),
                button(text("Today").size(14))
                    .padding(10)
                    .on_press(Message::SetDraftDate(Some(today_6pm))),
                button(text("Tomorrow").size(14))
                    .padding(10)
                    .on_press(Message::SetDraftDate(Some(tomorrow))),
                button(text("Next Week").size(14))
                    .padding(10)
                    .on_press(Message::SetDraftDate(Some(next_week))),
                button(text("Clear").size(14))
                    .padding(10)
                    .on_press(Message::SetDraftDate(None)),
                text(match &state.draft_due_date {
                    Some(d) => format!("✅ {}", d.format("%Y-%m-%d %H:%M")),
                    None => "".to_string(),
                })
                .size(14)
            ]
            .spacing(15)
            .align_y(Alignment::Center);

            let priority_chips = row![
                text("Priority:").size(16),
                button(text("🟢 Low").size(14))
                    .padding(10)
                    .on_press(Message::SetDraftPriority(Priority::Low)),
                button(text("🟡 Medium").size(14))
                    .padding(10)
                    .on_press(Message::SetDraftPriority(Priority::Medium)),
                button(text("🔴 High").size(14))
                    .padding(10)
                    .on_press(Message::SetDraftPriority(Priority::High)),
                text(format!("Selected: {:?}", state.draft_task_priority)).size(14)
            ]
            .spacing(15)
            .align_y(Alignment::Center);

            let submit_btn = button(text("🚀 Create Task").size(18))
                .padding(15)
                .width(Length::Fill)
                .on_press(Message::SubmitTask);

            let form = container(
                column![
                    input_title,
                    row![input_desc, input_duration].spacing(15),
                    priority_chips,
                    date_chips,
                    submit_btn
                ]
                .spacing(15),
            )
            .padding(25);

            let tasks_column = state.tasks.iter().filter(|t| !t.is_completed).fold(
                column![].spacing(15),
                |col, task| {
                    let mut details = String::new();
                    details.push_str(match task.priority {
                        Priority::Low => "\n🟢 Low Priority",
                        Priority::Medium => "\n🟡 Medium Priority",
                        Priority::High => "\n🔴 High Priority",
                    });
                    if let Some(d) = &task.description {
                        details.push_str(&format!(" | 📝 {}", d));
                    }
                    if let Some(dur) = task.duration_minutes {
                        details.push_str(&format!(" | ⏳ {} mins", dur));
                    }
                    if let Some(d) = &task.due_date {
                        details.push_str(&format!(" | ⏰ {}", d.format("%Y-%m-%d %H:%M")));
                    }

                    let check = row![
                        checkbox(task.is_completed)
                            .on_toggle(move |_| Message::ToggleTask(task.id)),
                        column![text(&task.title).size(20), text(details).size(15)]
                    ]
                    .spacing(15)
                    .align_y(Alignment::Start);

                    col.push(container(check).padding(15).width(Length::Fill))
                },
            );

            column![
                title,
                scrollable(tasks_column),
                Space::new().height(Length::Fill),
                form
            ]
            .spacing(20)
            .into()
        }
        Tab::Calendar => {
            let title = text("Calendar & Agenda").size(36);
            let upcoming_col = state
                .tasks
                .iter()
                .filter(|t| !t.is_completed && t.due_date.is_some())
                .fold(column![].spacing(15), |col, task| {
                    col.push(
                        text(format!(
                            "{} - Due: {}",
                            task.title,
                            task.due_date.unwrap().format("%Y-%m-%d %H:%M")
                        ))
                        .size(20),
                    )
                });

            let no_date_col = state
                .tasks
                .iter()
                .filter(|t| !t.is_completed && t.due_date.is_none())
                .fold(column![].spacing(15), |col, task| {
                    col.push(text(task.title.to_string()).size(20))
                });

            column![
                title,
                text("Scheduled Alarms").size(24),
                scrollable(upcoming_col),
                text("Unscheduled Tasks").size(24),
                scrollable(no_date_col)
            ]
            .spacing(20)
            .into()
        }
        Tab::Analytics => {
            let title = text("Analytics Overview").size(36);
            let total = state.tasks.len();
            let completed = state.tasks.iter().filter(|t| t.is_completed).count();
            let pending = total - completed;

            column![
                title,
                text(format!("Total Tasks Logged: {}", total)).size(24),
                text(format!("Tasks Completed: {}", completed)).size(24),
                text(format!("Tasks Pending: {}", pending)).size(24),
            ]
            .spacing(25)
            .into()
        }
    };

    let main_panel = container(active_content)
        .padding(50)
        .width(Length::Fill)
        .height(Length::Fill);

    row![sidebar, main_panel].into()
}
