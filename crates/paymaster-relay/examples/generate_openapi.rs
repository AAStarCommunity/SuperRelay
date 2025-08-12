// examples/generate_openapi.rs
// 演示 utoipa 自动生成 OpenAPI 文档的功能

use rundler_paymaster_relay::api_schemas::ApiDoc;
use utoipa::OpenApi;

fn main() {
    println!("🔧 生成 utoipa OpenAPI 文档演示");
    println!("=================================");
    println!();

    // 生成 OpenAPI 规范
    let openapi = ApiDoc::openapi();

    println!("📋 OpenAPI 文档信息:");
    println!("标题: {}", openapi.info.title);
    println!("版本: {}", openapi.info.version);
    if let Some(description) = &openapi.info.description {
        println!("描述: {}", description);
    }
    println!();

    // 显示服务器信息
    if let Some(servers) = &openapi.servers {
        println!("🌐 服务器列表:");
        for server in servers {
            println!(
                "  - {}: {}",
                server.url,
                server.description.as_deref().unwrap_or("")
            );
        }
        println!();
    }

    // 显示 API 路径
    println!("🛣️  API 路径:");
    for (path, _path_item) in &openapi.paths.paths {
        println!("  {}", path);
    }
    println!();

    // 显示数据模型
    if let Some(components) = &openapi.components {
        println!("📦 数据模型 (Schemas):");
        for schema_name in components.schemas.keys() {
            println!("  - {}", schema_name);
        }
        println!();
    }

    // 显示标签
    if let Some(tags) = &openapi.tags {
        println!("🏷️  API 标签:");
        for tag in tags {
            println!(
                "  - {}: {}",
                tag.name,
                tag.description.as_deref().unwrap_or("")
            );
        }
        println!();
    }

    // 生成完整的 JSON 输出
    match serde_json::to_string_pretty(&openapi) {
        Ok(json) => {
            println!("✅ OpenAPI JSON 生成成功!");
            println!("JSON 大小: {} 字节", json.len());

            // 保存到文件
            let output_path = "generated_openapi.json";
            if let Err(e) = std::fs::write(output_path, &json) {
                eprintln!("❌ 保存文件失败: {}", e);
            } else {
                println!("📄 已保存到: {}", output_path);
            }

            // 显示前几行 JSON 作为预览
            println!();
            println!("📖 JSON 预览 (前20行):");
            println!("{}", "─".repeat(50));
            for (i, line) in json.lines().take(20).enumerate() {
                println!("{:2}: {}", i + 1, line);
            }
            if json.lines().count() > 20 {
                println!("... (省略 {} 行)", json.lines().count() - 20);
            }
            println!("{}", "─".repeat(50));
        }
        Err(e) => {
            eprintln!("❌ JSON 序列化失败: {}", e);
            std::process::exit(1);
        }
    }

    println!();
    println!("🎉 utoipa OpenAPI 文档生成演示完成!");
    println!();
    println!("💡 使用方法:");
    println!("1. 启动 HTTP 服务器集成 Swagger UI");
    println!("2. 访问 /swagger-ui/ 查看交互式文档");
    println!("3. 访问 /api-doc/openapi.json 获取 JSON 规范");
}
