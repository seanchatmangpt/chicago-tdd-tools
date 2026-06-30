//! Integration tests for the A2A stub and harness.
//!
//! Requires: `cargo test -p chicago-tdd-mcp --features a2a`

#[cfg(feature = "a2a")]
mod a2a {
    use chicago_tdd_mcp::a2a::{
        assert::{assert_task_completed, assert_task_text_content},
        types::*,
        A2aStubAgent, A2aTaskHarness,
    };

    fn sample_card() -> AgentCard {
        AgentCard {
            name: "test-agent".into(),
            description: "A stub agent for testing".into(),
            url: String::new(),
            skills: vec![AgentSkill {
                id: "echo".into(),
                name: "Echo".into(),
                description: "Echoes messages".into(),
            }],
            version: "1.0.0".into(),
        }
    }

    fn hello_task(id: &str) -> Task {
        Task {
            id: id.into(),
            state: TaskState::Completed,
            messages: vec![Message {
                role: MessageRole::Agent,
                parts: vec![Part::Text(TextPart { text: "Hello back!".into() })],
            }],
            error: None,
        }
    }

    fn send_params(id: &str, text: &str) -> TaskSendParams {
        TaskSendParams {
            id: id.into(),
            message: Message {
                role: MessageRole::User,
                parts: vec![Part::Text(TextPart { text: text.into() })],
            },
            session_id: None,
        }
    }

    #[tokio::test]
    async fn stub_responds_to_task_send() {
        let stub = A2aStubAgent::builder(sample_card())
            .on_task_send("hello", hello_task("task-1"))
            .build()
            .await
            .expect("failed to build stub");

        let harness = A2aTaskHarness::new(stub.url());
        let task = harness
            .send_task(send_params("task-1", "hello"))
            .await
            .expect("send_task failed");

        assert_task_completed(&task);
        assert_task_text_content(&task, "Hello back!");
        stub.shutdown();
    }

    #[tokio::test]
    async fn stub_records_received_tasks() {
        let stub = A2aStubAgent::builder(sample_card())
            .default_response(hello_task("task-2"))
            .build()
            .await
            .expect("failed to build stub");

        let harness = A2aTaskHarness::new(stub.url());
        harness.send_task(send_params("task-2", "ping")).await.expect("send failed");
        harness.send_task(send_params("task-3", "pong")).await.expect("send failed");

        let recorded = stub.recorded_tasks().await;
        assert_eq!(recorded.len(), 2, "expected 2 recorded tasks");
        stub.assert_received_task_with_text("ping").await;
        stub.assert_received_task_with_text("pong").await;
        stub.shutdown();
    }

    #[tokio::test]
    async fn harness_fetches_agent_card() {
        let stub = A2aStubAgent::builder(sample_card())
            .build()
            .await
            .expect("failed to build stub");

        let harness = A2aTaskHarness::new(stub.url());
        let card = harness.fetch_agent_card().await.expect("fetch_agent_card failed");

        assert_eq!(card.name, "test-agent");
        assert_eq!(card.skills.len(), 1);
        assert_eq!(card.skills[0].id, "echo");
        stub.shutdown();
    }

    #[tokio::test]
    async fn cancel_task_returns_success() {
        let stub = A2aStubAgent::builder(sample_card())
            .build()
            .await
            .expect("failed to build stub");

        let harness = A2aTaskHarness::new(stub.url());
        let result = harness.cancel_task("task-99").await.expect("cancel failed");
        assert_eq!(result["canceled"], true);
        stub.shutdown();
    }
}
