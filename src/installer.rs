use std::env;

use crate::win;
use crate::win::taskschd::{ExecAction, LogonTrigger, RegistrationTrigger, TaskService, Trigger};

pub fn install() -> anyhow::Result<()> {
    let task_name = "Mona Startup";
    let user_name = win::get_user_name()?;

    let exe_path = env::current_exe()?;
    let exe_args = "run";

    let task_service = TaskService::new()?;

    let root_folder = task_service.folder("\\")?;
    let task = task_service.new_task()?;

    match root_folder.delete_task(task_name) {
        Ok(_) => println!("Removed existing task '{}'", task_name),
        Err(_) => println!("Task '{}' does not exist, will be created", task_name),
    }

    task.registration_info()?.set_author(&user_name)?;
    task.settings()?.set_start_when_available(true)?;

    let logon_trigger: LogonTrigger = task.triggers()?.create()?;

    logon_trigger.set_id("LogonTrigger")?;
    logon_trigger.set_user_id(&user_name)?;

    let registration_trigger: RegistrationTrigger = task.triggers()?.create()?;

    registration_trigger.set_id("RegistrationTrigger")?;

    let action: ExecAction = task.actions()?.create()?;

    action.set_path(exe_path.to_str().unwrap())?;
    action.set_arguments(exe_args)?;

    root_folder.register_task(task_name, &task, &user_name)?;

    println!("New task '{}' was successfully installed 🎉", task_name);

    Ok(())
}

pub fn uninstall() -> anyhow::Result<()> {
    let task_name = "Mona Startup";
    let task_service = TaskService::new()?;

    let root_folder = task_service.folder("\\")?;
    match root_folder.delete_task(task_name) {
        Ok(_) => println!("Task '{}' was successfully uninstalled 🎉", task_name),
        Err(_) => println!("Task '{}' is not installed", task_name),
    }

    Ok(())
}
