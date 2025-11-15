// Expert-level tests for testcontainers module

#[cfg(all(feature = "testcontainers", test))]
mod expert_tests {
    mod common {
        include!("../test_common.inc");
    }
    use chicago_tdd_tools::test;
    use chicago_tdd_tools::testcontainers::*;
    use common::require_docker;
    
    // Macros need to be used with full path in nested modules
    macro_rules! assert_ok {
        ($($args:tt)*) => {
            chicago_tdd_tools::assert_ok!($($args)*)
        };
    }

    const ALPINE_IMAGE: &str = "alpine";
    const ALPINE_TAG: &str = "latest";
    const NGINX_IMAGE: &str = "nginx";
    const NGINX_TAG: &str = "latest";

    test!(warmup_image_pull, {
        // Arrange: Set up Docker and images to pull
        require_docker();
        let client = ContainerClient::new();
        let images = vec![(ALPINE_IMAGE, ALPINE_TAG), (NGINX_IMAGE, NGINX_TAG)];

        // Act: Pull images by creating containers
        for (image, tag) in images {
            let container_result = GenericContainer::new(client.client(), image, tag);
            
            // Assert: Verify each image pulls successfully
            assert_ok!(
                &container_result,
                &format!("Image {}:{} should be pulled successfully", image, tag)
            );
            let _container = container_result.expect("Container should be created");
        }
    });
}
