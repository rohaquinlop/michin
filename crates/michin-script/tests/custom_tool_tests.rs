use michin_script::{ScriptDef, ScriptEngine};
use std::path::PathBuf;

#[test]
fn test_register_custom_tool() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "echo-tool".into(),
        location: PathBuf::from("echo.rhai"),
        source: r#"
            tool.register("echo", #{
                description: "Echo back the input.",
                parameters: #{
                    type: "object",
                    properties: #{
                        text: #{ type: "string", description: "Text to echo" }
                    },
                    required: ["text"]
                }
            });

            fn execute(args) {
                args.text
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");
    assert_eq!(tools[0].description, "Echo back the input.");
    assert_eq!(tools[0].script_name, "echo-tool");

    // Verify parameters schema was converted to JSON
    let params = &tools[0].parameters;
    assert_eq!(params["type"], "object");
    assert!(params["properties"]["text"].is_object());
}

#[test]
fn test_execute_custom_tool_returns_string() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "greet".into(),
        location: PathBuf::from("greet.rhai"),
        source: r#"
            tool.register("greet", #{
                description: "Greet someone.",
                parameters: #{
                    type: "object",
                    properties: #{
                        name: #{ type: "string", description: "Name to greet" }
                    },
                    required: ["name"]
                }
            });

            fn execute(args) {
                "Hello, " + args.name + "!"
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    let tool_def = &engine.registered_tools()[0];
    let args = serde_json::json!({"name": "World"});
    let result = engine.eval_tool_execute(tool_def, &args).unwrap();

    assert_eq!(result.content, "Hello, World!");
    assert!(!result.is_error);
}

#[test]
fn test_execute_custom_tool_returns_map() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "calc".into(),
        location: PathBuf::from("calc.rhai"),
        source: r#"
            tool.register("calc", #{
                description: "Add two numbers.",
                parameters: #{
                    type: "object",
                    properties: #{
                        a: #{ type: "number" },
                        b: #{ type: "number" }
                    },
                    required: ["a", "b"]
                }
            });

            fn execute(args) {
                let sum = args.a + args.b;
                #{ content: sum.to_string(), is_error: false }
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    let tool_def = &engine.registered_tools()[0];
    let args = serde_json::json!({"a": 3, "b": 4});
    let result = engine.eval_tool_execute(tool_def, &args).unwrap();

    assert_eq!(result.content, "7");
    assert!(!result.is_error);
}

#[test]
fn test_execute_custom_tool_error_result() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "fail-tool".into(),
        location: PathBuf::from("fail.rhai"),
        source: r#"
            tool.register("fail", #{
                description: "Always fails.",
                parameters: #{
                    type: "object",
                    properties: #{},
                    required: []
                }
            });

            fn execute(_args) {
                #{ content: "something broke", is_error: true }
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    let tool_def = &engine.registered_tools()[0];
    let args = serde_json::json!({});
    let result = engine.eval_tool_execute(tool_def, &args).unwrap();

    assert_eq!(result.content, "something broke");
    assert!(result.is_error);
}

#[test]
fn test_custom_tool_uses_exec() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "exec-tool".into(),
        location: PathBuf::from("exec.rhai"),
        source: r#"
            tool.register("echo_cmd", #{
                description: "Run echo via subprocess.",
                parameters: #{
                    type: "object",
                    properties: #{
                        text: #{ type: "string" }
                    },
                    required: ["text"]
                }
            });

            fn execute(args) {
                let result = exec("echo", [args.text]);
                result.stdout
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    let tool_def = &engine.registered_tools()[0];
    let args = serde_json::json!({"text": "hello from exec"});
    let result = engine.eval_tool_execute(tool_def, &args).unwrap();

    assert!(result.content.contains("hello from exec"));
    assert!(!result.is_error);
}

#[test]
fn test_custom_tool_sequential_mode() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "seq-tool".into(),
        location: PathBuf::from("seq.rhai"),
        source: r#"
            tool.register("seq_op", #{
                description: "A sequential operation.",
                execution_mode: "sequential",
                parameters: #{
                    type: "object",
                    properties: #{},
                    required: []
                }
            });

            fn execute(_args) {
                "done"
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0].execution_mode,
        michin_agent_core::types::ToolExecutionMode::Sequential
    );
}

#[test]
fn test_multiple_tools_in_one_script() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "multi-tools".into(),
        location: PathBuf::from("multi.rhai"),
        source: r#"
            tool.register("upper", #{
                description: "Uppercase text.",
                parameters: #{
                    type: "object",
                    properties: #{ text: #{ type: "string" } },
                    required: ["text"]
                }
            });

            tool.register("lower", #{
                description: "Lowercase text.",
                parameters: #{
                    type: "object",
                    properties: #{ text: #{ type: "string" } },
                    required: ["text"]
                }
            });

            fn execute(args) {
                // Both use the same execute — in practice you'd
                // distinguish by the registered name.
                args.text
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "upper");
    assert_eq!(tools[1].name, "lower");
}

#[test]
fn test_custom_tool_coexists_with_hooks() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "hybrid".into(),
        location: PathBuf::from("hybrid.rhai"),
        source: r#"
            tool.register("my_tool", #{
                description: "A custom tool.",
                parameters: #{
                    type: "object",
                    properties: #{},
                    required: []
                }
            });

            tool.before("bash", |ctx| {
                if ctx.args.command.contains("danger") {
                    return #{ blocked: true, reason: "no danger" };
                }
            });

            fn execute(_args) {
                "custom tool result"
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    // Tool is registered
    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "my_tool");

    // Hook is also registered
    let args = serde_json::json!({"command": "do danger"});
    let result = engine.eval_before("bash", &args).unwrap();
    assert!(matches!(
        result,
        michin_script::BeforeHookResult::Block { .. }
    ));

    // Custom tool executes
    let tool_args = serde_json::json!({});
    let result = engine.eval_tool_execute(&tools[0], &tool_args).unwrap();
    assert_eq!(result.content, "custom tool result");
}

// ── Integration: actual tool files from ~/.michin/tools/ ──

#[test]
fn test_web_search_tool_file_loads() {
    let path = std::path::Path::new(&std::env::var("HOME").unwrap_or("/tmp".into()))
        .join(".michin")
        .join("tools")
        .join("web-search.rhai");
    if !path.exists() {
        return; // skip if file not present
    }
    let engine = ScriptEngine::new(PathBuf::from("."));
    let source = std::fs::read_to_string(&path).unwrap();
    let script = ScriptDef {
        name: "web-search".into(),
        location: path,
        source,
    };
    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "web_search");
    assert!(tools[0].description.contains("Search the web"));
    assert!(tools[0].parameters["properties"]["query"].is_object());
}

#[test]
fn test_web_fetch_tool_file_loads() {
    let path = std::path::Path::new(&std::env::var("HOME").unwrap_or("/tmp".into()))
        .join(".michin")
        .join("tools")
        .join("web-fetch.rhai");
    if !path.exists() {
        return;
    }
    let engine = ScriptEngine::new(PathBuf::from("."));
    let source = std::fs::read_to_string(&path).unwrap();
    let script = ScriptDef {
        name: "web-fetch".into(),
        location: path,
        source,
    };
    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "web_fetch");
    assert!(tools[0].description.contains("Fetch a URL"));
    assert!(tools[0].parameters["properties"]["url"].is_object());
    assert!(tools[0].parameters["properties"]["workdir"].is_object());
}

// ── Edge cases and regression tests ──

#[test]
fn test_custom_tool_const_with_exec_comparison() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "const-exec".into(),
        location: PathBuf::from("const_exec.rhai"),
        source: r#"
            const MY_CMD = "echo";

            tool.register("const_exec", #{
                description: "Test const + exec.",
                parameters: #{
                    type: "object",
                    properties: #{},
                    required: []
                }
            });

            fn execute(_args) {
                let r = exec(MY_CMD, ["hello"]);
                if r.exit_code == 0 {
                    r.stdout
                } else {
                    #{ content: "failed", is_error: true }
                }
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 1);
    let result = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({}))
        .unwrap();
    assert!(result.content.contains("hello"));
    assert!(!result.is_error);
}

#[test]
fn test_is_error_integer_coercion() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "int-error".into(),
        location: PathBuf::from("int_error.rhai"),
        source: r#"
            tool.register("int_err", #{
                description: "Test is_error as int.",
                parameters: #{
                    type: "object",
                    properties: #{},
                    required: []
                }
            });

            fn execute(_args) {
                #{ content: "bad", is_error: 1 }
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    let result = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({}))
        .unwrap();
    assert_eq!(result.content, "bad");
    assert!(result.is_error);
}

#[test]
fn test_no_duplicate_hooks_after_tool_execute() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "dup-test".into(),
        location: PathBuf::from("dup.rhai"),
        source: r#"
            tool.register("dup_tool", #{
                description: "Dup test.",
                parameters: #{
                    type: "object",
                    properties: #{},
                    required: []
                }
            });

            tool.before("bash", |ctx| {
                // guard hook
            });

            tui.status("dup:status", |ctx| { "on" });

            fn execute(_args) { "ok" }
        "#
        .into(),
    };

    engine.load(&script).unwrap();

    // 1 tool, 1 before-hook, 1 status
    assert_eq!(engine.registered_tools().len(), 1);
    let statuses_before = engine.eval_tui_statuses();
    assert_eq!(statuses_before.len(), 1);

    // Execute the tool — triggers AST re-eval
    let tools = engine.registered_tools();
    let _ = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({}))
        .unwrap();

    // Still 1 tool (not duplicated)
    assert_eq!(engine.registered_tools().len(), 1);
    // Status handlers still 1 (not duplicated)
    let statuses_after = engine.eval_tui_statuses();
    assert_eq!(statuses_after.len(), 1);
    // Before hooks still 1 (not duplicated — a safe hook, so eval_before should pass)
    let args = serde_json::json!({"command": "ls"});
    let result = engine.eval_before("bash", &args).unwrap();
    assert!(matches!(result, michin_script::BeforeHookResult::Allow));
}

#[test]
fn test_init_error_is_surfaced() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "bad-const".into(),
        location: PathBuf::from("bad.rhai"),
        source: r#"
            const FOO = undefined_variable;

            tool.register("bad", #{
                description: "Bad tool.",
                parameters: #{ type: "object", properties: #{}, required: [] }
            });

            fn execute(_args) { FOO }
        "#
        .into(),
    };

    // The error surfaces at load time since top-level eval fails.
    let err = engine.load(&script).unwrap_err();
    assert!(
        err.contains("undefined_variable"),
        "expected const error, got: {err}"
    );
    assert!(engine.registered_tools().is_empty());
}

#[test]
fn test_execute_error_is_surfaced() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "undef-in-exec".into(),
        location: PathBuf::from("undef.rhai"),
        source: r#"
            tool.register("undef_tool", #{
                description: "Undefined ref in execute.",
                parameters: #{ type: "object", properties: #{}, required: [] }
            });

            fn execute(_args) {
                let x = this_does_not_exist;
                x
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    assert_eq!(tools.len(), 1);

    let err = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({}))
        .unwrap_err();
    assert!(
        err.contains("execute() error") || err.contains("not found"),
        "expected error, got: {err}"
    );
}

#[test]
fn test_exec_integer_type_consistency() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "exec-types".into(),
        location: PathBuf::from("exec_types.rhai"),
        source: r#"
            tool.register("exec_types", #{
                description: "Test exec types.",
                parameters: #{
                    type: "object",
                    properties: #{},
                    required: []
                }
            });

            fn execute(_args) {
                let r = exec("echo", ["test"]);
                // Ensure exit_code works with all integer comparisons
                if r.exit_code == 0 { } else { return "should not reach"; }
                if r.exit_code != 1 { } else { return "should not reach"; }
                if r.exit_code > -1 { } else { return "should not reach"; }
                if r.exit_code < 100 { } else { return "should not reach"; }
                if r.exit_code >= 0 { } else { return "should not reach"; }
                if r.exit_code <= 0 { } else { return "should not reach"; }
                r.stdout
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    let result = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({}))
        .unwrap();
    assert!(result.content.contains("test"));
    assert!(!result.is_error);
}

#[test]
fn test_array_iteration_with_push() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "arr-push".into(),
        location: PathBuf::from("arr_push.rhai"),
        source: r#"
            tool.register("arr_test", #{
                description: "Test array push in for loop.",
                parameters: #{
                    type: "object",
                    properties: #{ items: #{ type: "array" } },
                    required: ["items"]
                }
            });

            fn execute(args) {
                let out = ["base"];
                for item in args.items { out.push(item); }
                out.len().to_string()
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    let result = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({"items": ["a", "b", "c"]}))
        .unwrap();
    assert_eq!(result.content, "4");
}

#[test]
fn test_map_contains_check_for_optional_args() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "map-contains".into(),
        location: PathBuf::from("map_contains.rhai"),
        source: r#"
            tool.register("opt_test", #{
                description: "Test optional arg handling.",
                parameters: #{
                    type: "object",
                    properties: #{
                        name: #{ type: "string" },
                        count: #{ type: "number" }
                    },
                    required: ["name"]
                }
            });

            fn execute(args) {
                let name = if args.contains("name") { args.name } else { "anon" };
                let count = if args.contains("count") { args.count.to_string() } else { "0" };
                name + ":" + count
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();
    let tools = engine.registered_tools();

    // With both args
    let r1 = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({"name": "x", "count": 5}))
        .unwrap();
    assert_eq!(r1.content, "x:5");

    // With only required
    let r2 = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({"name": "y"}))
        .unwrap();
    assert_eq!(r2.content, "y:0");
}

#[test]
fn test_str_trim_returns_value_not_unit() {
    let engine = ScriptEngine::new(PathBuf::from("."));
    let script = ScriptDef {
        name: "trim-test".into(),
        location: PathBuf::from("trim_test.rhai"),
        source: r#"
            tool.register("trim_test", #{
                description: "Test str_trim returns a string.",
                parameters: #{
                    type: "object",
                    properties: #{
                        text: #{ type: "string" }
                    },
                    required: ["text"]
                }
            });

            fn execute(args) {
                let t = str_trim(args.text);
                t + ":" + t.len()
            }
        "#
        .into(),
    };

    engine.load(&script).unwrap();
    let tools = engine.registered_tools();
    let result = engine
        .eval_tool_execute(&tools[0], &serde_json::json!({"text": "  hello  "}))
        .unwrap();
    assert_eq!(result.content, "hello:5");
}
