// Expert-level tests for testcontainers module

#[cfg(all(feature = "testcontainers", test))]
mod expert_tests {
    mod common {
        include!("../common.rs");
    }
    use chicago_tdd_tools::chicago_test;
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::testcontainers::*;
    use common::require_docker;

    const ALPINE_IMAGE: &str = "alpine";
    const ALPINE_TAG: &str = "latest";
    const NGINX_IMAGE: &str = "nginx";
    const NGINX_TAG: &str = "latest";

    chicago_test!(warmup_image_pull, {
        require_docker();
        let client = ContainerClient::new();
        let images = vec![(ALPINE_IMAGE, ALPINE_TAG), (NGINX_IMAGE, NGINX_TAG)];

        for (image, tag) in images {
            let container_result = GenericContainer::new(client.client(), image, tag);
            assert_ok!(
                &container_result,
                &format!("Image {}:{} should be pulled successfully", image, tag)
            );
            let _container = container_result.expect("Container should be created");
        }
    });
}
