#!/usr/bin/env python3
"""
验证 CrossCopy 网络层迁移的脚本
检查从 WebSocket 到 libp2p 的迁移是否完成
"""

import os
import re
import sys
from pathlib import Path

def check_file_exists(file_path, description):
    """检查文件是否存在"""
    if os.path.exists(file_path):
        print(f"✅ {description}: {file_path}")
        return True
    else:
        print(f"❌ {description}: {file_path} (不存在)")
        return False

def check_file_content(file_path, patterns, description):
    """检查文件内容是否包含指定模式"""
    if not os.path.exists(file_path):
        print(f"❌ {description}: {file_path} (文件不存在)")
        return False
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        results = []
        for pattern, desc in patterns:
            if re.search(pattern, content, re.MULTILINE):
                results.append(f"  ✅ {desc}")
            else:
                results.append(f"  ❌ {desc}")
        
        print(f"📄 {description}: {file_path}")
        for result in results:
            print(result)
        
        return all("✅" in result for result in results)
    except Exception as e:
        print(f"❌ {description}: 读取文件失败 - {e}")
        return False

def main():
    print("CrossCopy 网络层迁移验证")
    print("=" * 40)
    
    # 检查项目根目录
    if not os.path.exists("Cargo.toml"):
        print("❌ 请在项目根目录运行此脚本")
        sys.exit(1)
    
    all_checks_passed = True
    
    # 1. 检查 Cargo.toml 依赖更新
    print("\n1. 检查依赖更新")
    cargo_patterns = [
        (r'libp2p.*=.*{.*version.*=.*"0\.53"', "libp2p 依赖已添加"),
        (r'"tcp"', "libp2p TCP 功能已配置"),
        (r'futures.*=.*"0\.3"', "futures 依赖已更新"),
    ]
    all_checks_passed &= check_file_content("Cargo.toml", cargo_patterns, "Cargo.toml 依赖配置")
    
    # 2. 检查配置结构更新
    print("\n2. 检查配置结构更新")
    config_patterns = [
        (r'pub enable_mdns: bool', "enable_mdns 字段已添加"),
        (r'pub mdns_discovery_interval: u64', "mdns_discovery_interval 字段已添加"),
        (r'pub enable_quic: bool', "enable_quic 字段已添加"),
        (r'pub idle_connection_timeout: u64', "idle_connection_timeout 字段已添加"),
    ]
    all_checks_passed &= check_file_content("src/config/mod.rs", config_patterns, "NetworkConfig 结构")
    
    # 3. 检查网络错误类型更新
    print("\n3. 检查网络错误类型更新")
    error_patterns = [
        (r'MdnsDiscoveryFailed\(String\)', "MdnsDiscoveryFailed 错误已添加"),
        (r'Libp2p\(String\)', "Libp2p 错误已添加"),
        (r'PeerNotFound\(String\)', "PeerNotFound 错误已添加"),
        (r'Transport\(String\)', "Transport 错误已添加"),
    ]
    all_checks_passed &= check_file_content("src/network/mod.rs", error_patterns, "网络错误类型")
    
    # 4. 检查连接结构更新
    print("\n4. 检查连接结构更新")
    connection_patterns = [
        (r'use libp2p::{PeerId, Multiaddr}', "libp2p 类型已导入"),
        (r'pub peer_id: Option<PeerId>', "peer_id 字段已添加"),
        (r'pub address: Option<Multiaddr>', "address 字段已添加"),
        (r'pub message_sender: Option<mpsc::UnboundedSender<Message>>', "message_sender 字段已添加"),
    ]
    all_checks_passed &= check_file_content("src/network/connection.rs", connection_patterns, "Connection 结构")
    
    # 5. 检查网络管理器更新
    print("\n5. 检查网络管理器更新")
    manager_patterns = [
        (r'use libp2p::', "libp2p 已导入"),
        (r'CrossCopyBehaviour', "CrossCopyBehaviour 已定义"),
        (r'mdns::Event::Discovered', "mDNS 发现事件处理"),
        (r'SwarmEvent::', "Swarm 事件处理"),
    ]
    all_checks_passed &= check_file_content("src/network/manager.rs", manager_patterns, "NetworkManager 实现")
    
    # 6. 检查文档更新
    print("\n6. 检查文档更新")
    doc_files = [
        ("doc/technical-specification.md", [
            (r'libp2p 协议栈', "技术规格已更新为 libp2p"),
            (r'mDNS 发现', "mDNS 发现机制已文档化"),
        ]),
        ("doc/api-reference.md", [
            (r'enable_mdns: bool', "API 文档已更新"),
            (r'mdns_discovery_interval: u64', "mDNS 配置已文档化"),
        ]),
        ("doc/architecture.md", [
            (r'libp2p.*点对点网络通信', "架构文档已更新"),
            (r'mDNS 自动节点发现', "mDNS 架构已说明"),
        ]),
    ]
    
    for file_path, patterns in doc_files:
        all_checks_passed &= check_file_content(file_path, patterns, f"文档: {file_path}")
    
    # 7. 检查测试和示例文件
    print("\n7. 检查测试和示例文件")
    test_files = [
        "tests/network_libp2p_test.rs",
        "examples/libp2p_network_demo.rs",
        "NETWORK_MIGRATION.md",
    ]
    
    for file_path in test_files:
        all_checks_passed &= check_file_exists(file_path, f"测试/示例文件")
    
    # 总结
    print("\n" + "=" * 40)
    if all_checks_passed:
        print("🎉 所有检查通过！网络层迁移已完成")
        print("\n迁移总结:")
        print("✅ WebSocket → libp2p 迁移完成")
        print("✅ mDNS 自动发现已实现")
        print("✅ 配置结构已更新")
        print("✅ 文档已同步更新")
        print("✅ 测试和示例已创建")
        print("\n下一步:")
        print("1. 运行 'cargo check' 检查编译")
        print("2. 运行 'cargo test' 执行测试")
        print("3. 运行 'cargo run --example libp2p_network_demo' 查看演示")
    else:
        print("⚠️  部分检查未通过，请检查上述问题")
        sys.exit(1)

if __name__ == "__main__":
    main()
