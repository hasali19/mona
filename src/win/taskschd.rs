use std::mem;
use std::ptr;

use anyhow::anyhow;
use winapi::{
    shared::{
        rpcdce::{RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE},
        winerror::{FAILED, HRESULT},
        wtypes::{VARIANT_FALSE, VARIANT_TRUE},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoUninitialize},
        objbase::COINIT_MULTITHREADED,
        taskschd::{
            IAction, IActionCollection, IExecAction, ILogonTrigger, IRegisteredTask,
            IRegistrationInfo, IRegistrationTrigger, ITaskDefinition, ITaskFolder, ITaskService,
            ITaskSettings, ITrigger, ITriggerCollection, TaskScheduler, TASK_ACTION_EXEC,
            TASK_ACTION_TYPE, TASK_CREATE_OR_UPDATE, TASK_LOGON_INTERACTIVE_TOKEN,
            TASK_TRIGGER_LOGON, TASK_TRIGGER_REGISTRATION, TASK_TRIGGER_TYPE2,
        },
        winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS},
    },
    Class, Interface,
};

use super::{BStr, Variant};

pub struct TaskService(*const ITaskService);

impl TaskService {
    pub fn new() -> anyhow::Result<Self> {
        let hr = unsafe { CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED) };
        if FAILED(hr) {
            return win_error_result(hr);
        }

        let hr = unsafe {
            CoInitializeSecurity(
                ptr::null_mut(),
                -1,
                ptr::null_mut(),
                ptr::null_mut(),
                RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
            )
        };

        if FAILED(hr) {
            unsafe { CoUninitialize() };
            return win_error_result(hr);
        }

        let mut service: *const ITaskService = ptr::null();
        let hr = unsafe {
            CoCreateInstance(
                &TaskScheduler::uuidof(),
                ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &ITaskService::uuidof(),
                &mut service as *mut _ as _,
            )
        };

        if FAILED(hr) {
            unsafe { CoUninitialize() };
            return win_error_result(hr);
        }

        let hr = unsafe {
            (*service).Connect(mem::zeroed(), mem::zeroed(), mem::zeroed(), mem::zeroed())
        };

        if FAILED(hr) {
            unsafe {
                (*service).Release();
                CoUninitialize();
            }
            return win_error_result(hr);
        }

        Ok(Self(service))
    }

    pub fn folder(&self, path: &str) -> anyhow::Result<TaskFolder> {
        let mut folder = ptr::null();
        let hr = unsafe {
            (*self.0).GetFolder(BStr::from_str(path).inner(), &mut folder as *mut _ as _)
        };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(TaskFolder(folder))
    }

    pub fn new_task(&self) -> anyhow::Result<TaskDefinition> {
        let mut task = ptr::null();
        let hr = unsafe { (*self.0).NewTask(0, &mut task as *mut _ as _) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(TaskDefinition(task))
    }
}

impl Drop for TaskService {
    fn drop(&mut self) {
        unsafe {
            (*self.0).Release();
            CoUninitialize();
        }
    }
}

pub struct TaskFolder(*const ITaskFolder);

impl TaskFolder {
    pub fn delete_task(&self, name: &str) -> anyhow::Result<()> {
        let hr = unsafe { (*self.0).DeleteTask(BStr::from_str(name).into(), 0) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }

    pub fn register_task(
        &self,
        name: &str,
        task: &TaskDefinition,
        user: &str,
    ) -> anyhow::Result<()> {
        let mut registered_task: *const IRegisteredTask = ptr::null();
        let hr = unsafe {
            (*self.0).RegisterTaskDefinition(
                BStr::from_str(name).inner(),
                task.0,
                TASK_CREATE_OR_UPDATE as _,
                Variant::bstr(&BStr::from_str(user)).inner(),
                Variant::null().inner(),
                TASK_LOGON_INTERACTIVE_TOKEN,
                Variant::bstr(&BStr::from_str("")).inner(),
                &mut registered_task as *mut _ as _,
            )
        };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }
}

impl Drop for TaskFolder {
    fn drop(&mut self) {
        unsafe { (*self.0).Release() };
    }
}

pub struct TaskDefinition(*const ITaskDefinition);

impl TaskDefinition {
    pub fn registration_info(&self) -> anyhow::Result<RegistrationInfo> {
        let mut reg_info = ptr::null();
        let hr = unsafe { (*self.0).get_RegistrationInfo(&mut reg_info as *mut _ as _) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(RegistrationInfo(reg_info))
    }

    pub fn settings(&self) -> anyhow::Result<TaskSettings> {
        let mut settings = ptr::null();
        let hr = unsafe { (*self.0).get_Settings(&mut settings as *mut _ as _) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(TaskSettings(settings))
    }

    pub fn triggers(&self) -> anyhow::Result<TriggerCollection> {
        let mut trigger_collection = ptr::null();
        let hr = unsafe { (*self.0).get_Triggers(&mut trigger_collection as *mut _ as _) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(TriggerCollection(trigger_collection))
    }

    pub fn actions(&self) -> anyhow::Result<ActionCollection> {
        let mut action_collection = ptr::null();
        let hr = unsafe { (*self.0).get_Actions(&mut action_collection as *mut _ as _) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(ActionCollection(action_collection))
    }
}

pub struct RegistrationInfo(*const IRegistrationInfo);

impl RegistrationInfo {
    pub fn set_author(&self, author: &str) -> anyhow::Result<()> {
        let hr = unsafe { (*self.0).put_Author(BStr::from_str(author).inner()) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }
}

pub struct TaskSettings(*const ITaskSettings);

impl TaskSettings {
    pub fn set_start_when_available(&self, value: bool) -> anyhow::Result<()> {
        let hr = unsafe {
            (*self.0).put_StartWhenAvailable(if value { VARIANT_TRUE } else { VARIANT_FALSE })
        };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }
}

pub struct TriggerCollection(*const ITriggerCollection);

impl TriggerCollection {
    pub fn create<T: Trigger>(&self) -> anyhow::Result<T> {
        let mut trigger: *const ITrigger = ptr::null();
        let hr = unsafe { (*self.0).Create(T::trigger_type(), &mut trigger as *mut _ as _) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        let mut actual_trigger = ptr::null();
        let res = unsafe {
            (*trigger).QueryInterface(&T::Interface::uuidof(), &mut actual_trigger as *mut _ as _)
        };

        unsafe { (*trigger).Release() };

        if FAILED(res) {
            return win_error_result(hr);
        }

        Ok(T::from_ptr(actual_trigger))
    }
}

pub trait Trigger {
    type Interface: winapi::Interface;

    fn trigger_type() -> TASK_TRIGGER_TYPE2;

    fn from_ptr(ptr: *const Self::Interface) -> Self;
    fn get_ptr(&self) -> *const ITrigger;

    fn set_id(&self, id: &str) -> anyhow::Result<()> {
        let hr = unsafe { (*self.get_ptr()).put_Id(BStr::from_str(id).into()) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }
}

pub struct LogonTrigger(*const ILogonTrigger);

impl LogonTrigger {
    pub fn set_user_id(&self, user_id: &str) -> anyhow::Result<()> {
        let hr = unsafe { (*self.0).put_UserId(BStr::from_str(user_id).into()) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }
}

impl Trigger for LogonTrigger {
    type Interface = ILogonTrigger;

    fn from_ptr(ptr: *const Self::Interface) -> Self {
        Self(ptr)
    }

    fn get_ptr(&self) -> *const ITrigger {
        self.0 as _
    }

    fn trigger_type() -> TASK_TRIGGER_TYPE2 {
        TASK_TRIGGER_LOGON
    }
}

pub struct RegistrationTrigger(*const IRegistrationTrigger);

impl Trigger for RegistrationTrigger {
    type Interface = IRegistrationTrigger;

    fn from_ptr(ptr: *const Self::Interface) -> Self {
        Self(ptr)
    }

    fn get_ptr(&self) -> *const ITrigger {
        self.0 as _
    }

    fn trigger_type() -> TASK_TRIGGER_TYPE2 {
        TASK_TRIGGER_REGISTRATION
    }
}

pub struct ActionCollection(*const IActionCollection);

impl ActionCollection {
    pub fn create<T: Action>(&self) -> anyhow::Result<T> {
        let mut action: *const IAction = ptr::null();
        let hr = unsafe { (*self.0).Create(T::action_type(), &mut action as *mut _ as _) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        let mut actual_action = ptr::null();
        let hr = unsafe {
            (*action).QueryInterface(&T::Interface::uuidof(), &mut actual_action as *mut _ as _)
        };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(T::from_ptr(actual_action))
    }
}

pub trait Action {
    type Interface: winapi::Interface;

    fn action_type() -> TASK_ACTION_TYPE;

    fn from_ptr(ptr: *const Self::Interface) -> Self;
    fn get_ptr(&self) -> *const ITrigger;
}

pub struct ExecAction(*const IExecAction);

impl ExecAction {
    pub fn set_path(&self, path: &str) -> anyhow::Result<()> {
        let hr = unsafe { (*self.0).put_Path(BStr::from_str(path).into()) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }

    pub fn set_arguments(&self, args: &str) -> anyhow::Result<()> {
        let hr = unsafe { (*self.0).put_Arguments(BStr::from_str(args).into()) };

        if FAILED(hr) {
            return win_error_result(hr);
        }

        Ok(())
    }
}

impl Action for ExecAction {
    type Interface = IExecAction;

    fn action_type() -> TASK_ACTION_TYPE {
        TASK_ACTION_EXEC
    }

    fn from_ptr(ptr: *const Self::Interface) -> Self {
        ExecAction(ptr)
    }

    fn get_ptr(&self) -> *const ITrigger {
        self.0 as _
    }
}

fn win_error_result<T>(hr: HRESULT) -> anyhow::Result<T> {
    let mut buf = [0; 512];
    let chars = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            ptr::null(),
            hr as _,
            0,
            buf.as_mut_ptr(),
            buf.len() as _,
            ptr::null_mut(),
        )
    };

    if chars == 0 {
        Err(anyhow!("Operation failed (0x{:x})", hr))
    } else {
        Err(anyhow!(
            "Operation failed (0x{:x}): {}",
            hr,
            String::from_utf16_lossy(&buf)
        ))
    }
}
