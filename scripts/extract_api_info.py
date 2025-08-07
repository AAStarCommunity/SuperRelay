#!/usr/bin/env python3
"""
SuperRelay API信息提取器
从Rust源代码中自动提取API端点、方法签名、参数类型等信息，生成OpenAPI规范
"""

import os
import re
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Set
from dataclasses import dataclass, asdict
import sys

@dataclass
class ApiEndpoint:
    method_name: str
    rpc_method: str
    description: str
    parameters: List[Dict]
    return_type: str
    file_path: str
    line_number: int

@dataclass
class ApiInfo:
    version: str
    endpoints: List[ApiEndpoint]
    data_types: Dict[str, Dict]

class RustCodeAnalyzer:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.rpc_methods: Dict[str, ApiEndpoint] = {}
        self.data_types: Dict[str, Dict] = {}
        
    def extract_version(self) -> str:
        """从Cargo.toml提取版本信息"""
        cargo_path = self.project_root / "Cargo.toml"
        if cargo_path.exists():
            with open(cargo_path, 'r') as f:
                content = f.read()
                match = re.search(r'version\s*=\s*"([^"]+)"', content)
                if match:
                    return match.group(1)
        return "0.1.0"
    
    def find_rpc_methods(self) -> None:
        """扫描所有Rust源文件，提取RPC方法定义"""
        rust_files = list(self.project_root.rglob("*.rs"))
        print(f"📂 扫描到 {len(rust_files)} 个Rust文件")
        
        for file_path in rust_files:
            try:
                self.analyze_rust_file(file_path)
            except Exception as e:
                print(f"⚠️  分析文件失败 {file_path}: {e}")
    
    def analyze_rust_file(self, file_path: Path) -> None:
        """分析单个Rust文件，提取API定义"""
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 查找 #[method(name = "...")] 注解
        method_pattern = r'#\[method\(name\s*=\s*"([^"]+)"\)\]\s*(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*([^{;]+))?'
        
        for match in re.finditer(method_pattern, content, re.MULTILINE):
            rpc_name = match.group(1)
            method_name = match.group(2)
            return_type = match.group(3).strip() if match.group(3) else "void"
            
            # 获取行号
            line_number = content[:match.start()].count('\n') + 1
            
            # 提取方法文档注释
            description = self.extract_method_doc(content, match.start())
            
            # 提取参数信息
            parameters = self.extract_method_parameters(content, match.group(0))
            
            endpoint = ApiEndpoint(
                method_name=method_name,
                rpc_method=rpc_name,
                description=description,
                parameters=parameters,
                return_type=return_type,
                file_path=str(file_path.relative_to(self.project_root)),
                line_number=line_number
            )
            
            self.rpc_methods[rpc_name] = endpoint
            print(f"✅ 发现API方法: {rpc_name} -> {method_name} ({file_path.name}:{line_number})")
    
    def extract_method_doc(self, content: str, method_start: int) -> str:
        """提取方法的文档注释"""
        lines = content[:method_start].split('\n')
        doc_lines = []
        
        # 从方法定义位置向上查找文档注释
        for line in reversed(lines[-10:]):  # 只查找前10行
            line = line.strip()
            if line.startswith('///'):
                doc_lines.append(line[3:].strip())
            elif line.startswith('//'):
                continue  # 跳过普通注释
            elif line and not line.startswith('#'):
                break  # 遇到非注释行停止
        
        return ' '.join(reversed(doc_lines)) if doc_lines else "No description available"
    
    def extract_method_parameters(self, content: str, method_def: str) -> List[Dict]:
        """提取方法参数信息"""
        # 简化的参数解析
        param_pattern = r'(\w+):\s*([^,)]+)'
        parameters = []
        
        for match in re.finditer(param_pattern, method_def):
            param_name = match.group(1)
            param_type = match.group(2).strip()
            
            # 跳过self参数
            if param_name == 'self':
                continue
                
            parameters.append({
                'name': param_name,
                'type': self.rust_type_to_json_schema(param_type),
                'required': not param_type.startswith('Option<')
            })
        
        return parameters
    
    def rust_type_to_json_schema(self, rust_type: str) -> Dict:
        """将Rust类型转换为JSON Schema类型"""
        rust_type = rust_type.strip()
        
        # 移除泛型包装
        if rust_type.startswith('RpcResult<'):
            rust_type = rust_type[10:-1]
        if rust_type.startswith('Option<'):
            rust_type = rust_type[7:-1]
        
        # 基本类型映射
        type_mapping = {
            'String': {'type': 'string'},
            'str': {'type': 'string'},
            'u64': {'type': 'integer', 'format': 'int64'},
            'u32': {'type': 'integer', 'format': 'int32'},
            'i64': {'type': 'integer', 'format': 'int64'},
            'i32': {'type': 'integer', 'format': 'int32'},
            'bool': {'type': 'boolean'},
            'f64': {'type': 'number', 'format': 'double'},
            'f32': {'type': 'number', 'format': 'float'},
            'Bytes': {'type': 'string', 'description': 'Hex-encoded bytes'},
            'Address': {'type': 'string', 'description': 'Ethereum address'},
            'B256': {'type': 'string', 'description': 'Hash value'},
            'U256': {'type': 'string', 'description': 'Large integer as string'},
        }
        
        # 数组类型
        if rust_type.startswith('Vec<'):
            item_type = rust_type[4:-1]
            return {
                'type': 'array',
                'items': self.rust_type_to_json_schema(item_type)
            }
        
        # 检查是否是已知类型
        if rust_type in type_mapping:
            return type_mapping[rust_type]
        
        # 自定义类型，生成引用
        if rust_type.startswith('Rpc'):
            return {'$ref': f'#/components/schemas/{rust_type}'}
        
        # 默认为object类型
        return {'type': 'object', 'description': f'Custom type: {rust_type}'}
    
    def find_data_structures(self) -> None:
        """查找数据结构定义（struct、enum等）"""
        rust_files = list(self.project_root.rglob("*.rs"))
        
        for file_path in rust_files:
            try:
                self.extract_data_structures(file_path)
            except Exception as e:
                print(f"⚠️  提取数据结构失败 {file_path}: {e}")
    
    def extract_data_structures(self, file_path: Path) -> None:
        """从Rust文件中提取struct和enum定义"""
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 查找struct定义
        struct_pattern = r'#\[derive\([^\]]*Serialize[^\]]*\)\]\s*pub struct (\w+)\s*\{([^}]+)\}'
        
        for match in re.finditer(struct_pattern, content, re.DOTALL):
            struct_name = match.group(1)
            struct_body = match.group(2)
            
            # 解析字段
            fields = self.parse_struct_fields(struct_body)
            
            self.data_types[struct_name] = {
                'type': 'object',
                'properties': fields,
                'file': str(file_path.relative_to(self.project_root))
            }
            
            print(f"📋 发现数据结构: {struct_name} (字段: {len(fields)})")
    
    def parse_struct_fields(self, struct_body: str) -> Dict:
        """解析struct字段"""
        fields = {}
        field_pattern = r'pub\s+(\w+):\s*([^,\n]+)'
        
        for match in re.finditer(field_pattern, struct_body):
            field_name = match.group(1)
            field_type = match.group(2).strip().rstrip(',')
            fields[field_name] = self.rust_type_to_json_schema(field_type)
        
        return fields
    
    def generate_openapi_spec(self) -> Dict:
        """生成OpenAPI规范"""
        version = self.extract_version()
        
        openapi_spec = {
            "openapi": "3.0.3",
            "info": {
                "title": "SuperRelay Auto-Generated API",
                "version": version,
                "description": f"自动从代码生成的API文档 (发现 {len(self.rpc_methods)} 个API方法)",
                "x-generated": {
                    "timestamp": subprocess.check_output(['date', '-u', '+%Y-%m-%dT%H:%M:%SZ']).decode().strip(),
                    "source": "自动代码分析",
                    "methods_found": len(self.rpc_methods),
                    "data_types_found": len(self.data_types)
                }
            },
            "servers": [
                {
                    "url": "http://localhost:3000",
                    "description": "开发环境 - SuperRelay Gateway"
                }
            ],
            "tags": [
                {
                    "name": "Paymaster API",
                    "description": "🎯 SuperRelay核心Paymaster功能 - Gas赞助和用户操作支持"
                },
                {
                    "name": "ERC-4337 API", 
                    "description": "📋 标准ERC-4337账户抽象API - 兼容所有AA钱包"
                },
                {
                    "name": "Rundler API",
                    "description": "🔧 Rundler扩展功能API - 高级bundler操作"
                },
                {
                    "name": "Debug API",
                    "description": "🐛 调试和测试API - 开发环境专用"
                },
                {
                    "name": "Admin API",
                    "description": "⚙️ 管理和配置API - 系统管理员专用"
                },
                {
                    "name": "Monitoring API",
                    "description": "📊 健康检查和监控API - 运维和监控"
                }
            ],
            "paths": {},
            "components": {
                "schemas": self.data_types
            }
        }
        
        # 按方法前缀分组 - 改进的分类系统
        method_groups = {
            # Paymaster 核心业务 API
            'sponsorUserOperation': 'Paymaster API',
            'pm_': 'Paymaster API',
            
            # 标准 ERC-4337 API
            'sendUserOperation': 'ERC-4337 API',
            'estimateUserOperationGas': 'ERC-4337 API',
            'getUserOperationByHash': 'ERC-4337 API',
            'getUserOperationReceipt': 'ERC-4337 API',
            'supportedEntryPoints': 'ERC-4337 API',
            'chainId': 'ERC-4337 API',
            'eth_': 'ERC-4337 API',
            
            # Rundler 扩展 API
            'maxPriorityFeePerGas': 'Rundler API',
            'dropLocalUserOperation': 'Rundler API', 
            'getMinedUserOperation': 'Rundler API',
            'getUserOperationStatus': 'Rundler API',
            'getPendingUserOperationBySenderNonce': 'Rundler API',
            'rundler_': 'Rundler API',
            
            # Debug 和测试 API
            'bundler_': 'Debug API',
            'debug_': 'Debug API',
            
            # 管理和配置 API
            'clearState': 'Admin API',
            'setTracking': 'Admin API',
            'admin_': 'Admin API',
            
            # 健康检查和监控 API
            'health': 'Monitoring API',
            'metrics': 'Monitoring API',
            'balance': 'Monitoring API'
        }
        
        # 生成路径
        for rpc_method, endpoint in self.rpc_methods.items():
            # 确定API组 - 优先精确匹配，然后前缀匹配
            group = 'Other'
            
            # 1. 先尝试精确匹配
            if rpc_method in method_groups:
                group = method_groups[rpc_method]
            else:
                # 2. 然后尝试前缀匹配
                for prefix, group_name in method_groups.items():
                    if rpc_method.startswith(prefix):
                        group = group_name
                        break
            
            # 生成路径信息
            path_info = {
                "post": {
                    "summary": f"{endpoint.method_name} - {rpc_method}",
                    "description": f"{endpoint.description}\n\n**源文件**: `{endpoint.file_path}:{endpoint.line_number}`",
                    "tags": [group],
                    "requestBody": {
                        "required": True,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "jsonrpc": {"type": "string", "example": "2.0"},
                                        "method": {"type": "string", "example": rpc_method},
                                        "params": {
                                            "type": "array",
                                            "items": {"type": "object"},
                                            "description": f"参数: {[p['name'] for p in endpoint.parameters]}"
                                        },
                                        "id": {"type": "integer", "example": 1}
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": f"Success response - {endpoint.return_type}",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "jsonrpc": {"type": "string", "example": "2.0"},
                                            "result": {"type": "object", "description": endpoint.return_type},
                                            "id": {"type": "integer"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            openapi_spec["paths"][f"/{rpc_method}"] = path_info
        
        return openapi_spec

def main():
    if len(sys.argv) > 1:
        project_root = sys.argv[1]
    else:
        project_root = os.getcwd()
    
    print(f"🔍 分析项目: {project_root}")
    analyzer = RustCodeAnalyzer(project_root)
    
    # 步骤1: 提取API方法
    print("\n📡 提取API方法...")
    analyzer.find_rpc_methods()
    
    # 步骤2: 提取数据结构
    print("\n📋 提取数据结构...")
    analyzer.find_data_structures()
    
    # 步骤3: 生成OpenAPI规范
    print("\n🛠️  生成OpenAPI规范...")
    openapi_spec = analyzer.generate_openapi_spec()
    
    # 步骤4: 保存文件
    output_file = Path(project_root) / "web-ui" / "swagger-ui" / "openapi.json"
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(openapi_spec, f, indent=2, ensure_ascii=False)
    
    print(f"\n✅ OpenAPI规范已生成: {output_file}")
    print(f"📊 统计信息:")
    print(f"   • API方法: {len(analyzer.rpc_methods)}")
    print(f"   • 数据类型: {len(analyzer.data_types)}")
    print(f"   • 项目版本: {openapi_spec['info']['version']}")
    
    # 显示发现的API方法
    if analyzer.rpc_methods:
        print(f"\n🎯 发现的API方法:")
        for rpc_method, endpoint in sorted(analyzer.rpc_methods.items()):
            print(f"   • {rpc_method:<30} -> {endpoint.method_name} ({endpoint.file_path})")

if __name__ == "__main__":
    main()