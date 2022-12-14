use crate::{
    lua::{
        loader::{encoding_func, get_matching_func, get_utilsfunc, http_func, payloads_func},
        network::http::Sender,
        output::vuln::AllReports,
    },
    RequestOpts,
};
use mlua::Lua;
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};
use thirtyfour::prelude::*;

#[derive(Clone)]
pub struct LuaLoader<'a> {
    output_dir: String,
    request: RequestOpts,
    bar: &'a indicatif::ProgressBar,
}

/// Start Lotus by adding the ProgressBar and http request options
impl<'a> LuaLoader<'a> {
    pub fn new(
        bar: &'a indicatif::ProgressBar,
        request: RequestOpts,
        output_dir: String,
    ) -> LuaLoader {
        LuaLoader {
            output_dir,
            request,
            bar,
        }
    }

    fn set_lua(&self, target_url: &str, lua: &Lua, driver: Option<Arc<Mutex<WebDriver>>>) {
        // Adding Lotus Lua Function
        get_utilsfunc(self.bar, &lua);
        get_matching_func(&lua);
        http_func(target_url, &lua);
        encoding_func(&lua);
        payloads_func(&lua);
        // HTTP Sender
        lua.globals()
            .set(
                "http",
                Sender::init(
                    self.request.headers.clone(),
                    self.request.proxy.clone(),
                    self.request.timeout,
                    self.request.redirects,
                ),
            )
            .unwrap();
        if !driver.is_none() {
            lua.globals()
                .set(
                    "openbrowser",
                    lua.create_function(move |_, url: String| {
                        futures::executor::block_on({
                            let driver = Arc::clone(driver.as_ref().unwrap());
                            async move {
                                driver.lock().unwrap().goto(url).await.unwrap();
                            }
                        });
                        Ok(())
                    })
                    .unwrap(),
                )
                .unwrap();
        }
    }
    fn write_report(&self, results: &str) {
        OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.output_dir)
            .expect("Could not open file")
            .write_all(format!("{}\n", results).as_str().as_bytes())
            .expect("Could not write to file");
    }

    pub async fn run_scan(
        &self,
        target_url: &str,
        driver: Option<Arc<Mutex<WebDriver>>>,
        script_code: &str,
        script_dir: &str,
    ) -> Result<(), mlua::Error> {
        let lua = Lua::new();
        // settings lua api
        self.set_lua(target_url, &lua, driver);
        lua.globals().set("SCRIPT_PATH", script_dir).unwrap();
        lua.globals()
            .set(
                "JOIN_SCRIPT_DIR",
                lua.create_function(|c_lua, new_path: String| {
                    let script_path = c_lua.globals().get::<_, String>("SCRIPT_PATH").unwrap();
                    let the_path = Path::new(&script_path);
                    Ok(the_path
                        .parent()
                        .unwrap()
                        .join(new_path)
                        .to_str()
                        .unwrap()
                        .to_string())
                })
                .unwrap(),
            )
            .unwrap();

        // Handle this error please
        let run_code = lua.load(script_code).exec_async().await;
        if run_code.is_err() {
            self.bar.inc(1);
            self.bar.println("Script Error");
            return run_code;
        }
        let main_func = lua.globals().get::<_, mlua::Function>("main");
        if main_func.is_err() {
            log::error!("[{}] there is no main function, Skipping ..", script_dir);
            self.bar.println(format!("[{}] there is no main function, Skipping ..", script_dir));
        } else {
            let run_scan = main_func
                .unwrap()
                .call_async::<_, mlua::Value>(target_url)
                .await;
            self.bar.inc(1);
            if run_scan.is_err() {
                log::error!(
                    "[{}] Script Error : {:?}",
                    script_dir,
                    run_scan.unwrap_err()
                );
                self.bar.println("Script ERROR");
            } else {
                let script_report = lua.globals().get::<_, AllReports>("Reports").unwrap();
                if !script_report.reports.is_empty() {
                    let results = serde_json::to_string(&script_report.reports).unwrap();
                    log::debug!(
                        "[{}] Report Length {}",
                        script_dir,
                        script_report.reports.len()
                    );
                    self.write_report(&results);
                } else {
                    log::debug!("[{}] Script report is empty", script_dir);
                }
            }
        }
        Ok(())
    }
}
