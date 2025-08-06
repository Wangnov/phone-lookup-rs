use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("开始初始化 Tauri 应用...");

    let result = tauri::Builder::default()
        .setup(|app| {
            eprintln!("Tauri 应用设置完成");
            
            // 获取资源文件路径
            let resource_path = app.path().resource_dir()
                .expect("获取资源目录失败")
                .join("_up_")
                .join("phone.dat");
            
            eprintln!("尝试从资源路径加载数据文件: {:?}", resource_path);
            
            // 初始化PhoneData实例
            let phone_data = match phone_lookup_rs::PhoneData::from_file(&resource_path.to_string_lossy()) {
                Ok(data) => {
                    eprintln!("数据库初始化成功，记录数: {}", data.get_total_records());
                    std::sync::Arc::new(data)
                }
                Err(e) => {
                    eprintln!("从资源路径初始化数据库失败: {}", e);
                    // 尝试备用路径（开发环境）
                    eprintln!("尝试备用路径...");
                    match phone_lookup_rs::PhoneData::new() {
                        Ok(data) => {
                            eprintln!("从备用路径加载成功，记录数: {}", data.get_total_records());
                            std::sync::Arc::new(data)
                        }
                        Err(backup_e) => {
                            eprintln!("备用路径也失败: {}", backup_e);
                            std::process::exit(1);
                        }
                    }
                }
            };
            
            // 将 phone_data 存储到应用状态中
            app.manage(phone_data);
            
            // 在开发模式下启用日志插件
            #[cfg(debug_assertions)]
            {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            phone_lookup_rs::tauri_commands::query_phone,
            phone_lookup_rs::tauri_commands::query_phones_batch,
            phone_lookup_rs::tauri_commands::get_app_info,
            phone_lookup_rs::tauri_commands::clear_cache,
            phone_lookup_rs::tauri_commands::set_cache_size
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(_) => eprintln!("Tauri 应用正常退出"),
        Err(e) => eprintln!("Tauri 应用启动失败: {}", e),
    }
}
