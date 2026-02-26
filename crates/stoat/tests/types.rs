use stoat::types::{AutumnResponse, BuildInformation, CaptchaFeature, Feature, RatelimitFailure, StoatConfig, StoatFeatures, Tag, VoiceFeature, VoiceNode};
pub use stoat_database::events::{
    client::{EventV1, Ping},
    server::ClientMessage,
};
pub use stoat_models::v0::*;
pub use stoat_permissions::{
    ChannelPermission, DataPermissionsValue, Override, OverrideField, PermissionValue,
    UserPermission,
};

#[test]
fn test_captcha_feature_deserialize() {
    let captcha_feature: CaptchaFeature = CaptchaFeature {
        enabled: true,
        key: "test".to_string(),
    };
    let json = serde_json::to_string(&captcha_feature).unwrap();
    assert_eq!(json, "{\"enabled\":true,\"key\":\"test\"}");
}

#[test]
fn test_captcha_feature_serialize() {
    let json = "{\"enabled\":true,\"key\":\"test\"}";
    let captcha_feature: CaptchaFeature = CaptchaFeature {
        enabled: true,
        key: "test".to_string(),
    };
    let captcha_feature_json = serde_json::from_str::<CaptchaFeature>(json).unwrap();
    assert_eq!(captcha_feature_json, captcha_feature);
}

#[test]
fn test_feature_deserialize() {
    let feature: Feature = Feature {
        enabled: true,
        url: "test".to_string(),
    };
    let json = serde_json::to_string(&feature).unwrap();
    assert_eq!(json, "{\"enabled\":true,\"url\":\"test\"}");
}

#[test]
fn test_feature_serialize() {
    let json = "{\"enabled\":true,\"url\":\"test\"}";
    let feature: Feature = Feature {
        enabled: true,
        url: "test".to_string(),
    };
    let feature_json = serde_json::from_str::<Feature>(json).unwrap();
    assert_eq!(feature_json, feature);
}

#[test]
fn test_voice_node_deserialize() {
    let voice_node: VoiceNode = VoiceNode {
        name: "test".to_string(),
        lat: 0.0,
        lon: 0.0,
        public_url: "test".to_string()
    };
    let json = serde_json::to_string(&voice_node).unwrap();
    assert_eq!(json, "{\"name\":\"test\",\"lat\":0.0,\"lon\":0.0,\"public_url\":\"test\"}");
}

#[test]
fn test_voice_node_serialize() {
    let json = "{\"name\":\"test\",\"lat\":0.0,\"lon\":0.0,\"public_url\":\"test\"}";
    let voice_node: VoiceNode = VoiceNode {
        name: "test".to_string(),
        lat: 0.0,
        lon: 0.0,
        public_url: "test".to_string()
    };
    let voice_node_json = serde_json::from_str::<VoiceNode>(json).unwrap();
    assert_eq!(voice_node_json, voice_node);
}

#[test]
fn test_voice_feature_deserialize() {
    let voice_node: VoiceFeature = VoiceFeature {
        enabled: true,
        nodes: vec![]
    };
    let json = serde_json::to_string(&voice_node).unwrap();
    assert_eq!(json, "{\"enabled\":true,\"nodes\":[]}");
}

#[test]
fn test_voice_feature_serialize() {
    let json = "{\"enabled\":true,\"nodes\":[]}";
    let voice_node: VoiceFeature = VoiceFeature {
        enabled: true,
        nodes: vec![]
    };
    let voice_node_json = serde_json::from_str::<VoiceFeature>(json).unwrap();
    assert_eq!(voice_node_json, voice_node);
}

#[test]
fn test_stoat_features_deserialize() {
    let stoat_features = StoatFeatures {
        captcha: CaptchaFeature {
            enabled: true,
            key: "test".to_string(),
        },
        email: false,
        invite_only: false,
        autumn: Feature {
            enabled: true,
            url: "test".to_string(),
        },
        january: Feature {
            enabled: true,
            url: "test".to_string(),
        },
        livekit: VoiceFeature {
            enabled: true,
            nodes: vec![]
        }
    };
    let json = serde_json::to_string(&stoat_features).unwrap();
    assert_eq!(json, "{\"captcha\":{\"enabled\":true,\"key\":\"test\"},\"email\":false,\"invite_only\":false,\"autumn\":{\"enabled\":true,\"url\":\"test\"},\"january\":{\"enabled\":true,\"url\":\"test\"},\"livekit\":{\"enabled\":true,\"nodes\":[]}}");
}

#[test]
fn test_stoat_features_serialize() {
    let json = "{\"captcha\":{\"enabled\":true,\"key\":\"test\"},\"email\":false,\"invite_only\":false,\"autumn\":{\"enabled\":true,\"url\":\"test\"},\"january\":{\"enabled\":true,\"url\":\"test\"},\"livekit\":{\"enabled\":true,\"nodes\":[]}}";
    let stoat_features = StoatFeatures {
        captcha: CaptchaFeature {
            enabled: true,
            key: "test".to_string(),
        },
        email: false,
        invite_only: false,
        autumn: Feature {
            enabled: true,
            url: "test".to_string(),
        },
        january: Feature {
            enabled: true,
            url: "test".to_string(),
        },
        livekit: VoiceFeature {
            enabled: true,
            nodes: vec![]
        }
    };
    let stoat_features_json = serde_json::from_str::<StoatFeatures>(json).unwrap();
    assert_eq!(stoat_features_json, stoat_features);
}

#[test]
fn test_build_information_deserialize() {
    let build_information = BuildInformation {
        commit_sha: "test".to_string(),
        commit_timestamp: "test".to_string(),
        semver: "test".to_string(),
        origin_url: "test".to_string(),
        timestamp: "test".to_string()
    };
    let json = serde_json::to_string(&build_information).unwrap();
    assert_eq!(json, "{\"commit_sha\":\"test\",\"commit_timestamp\":\"test\",\"semver\":\"test\",\"origin_url\":\"test\",\"timestamp\":\"test\"}");
}

#[test]
fn test_build_information_serialize() {
    let json = "{\"commit_sha\":\"test\",\"commit_timestamp\":\"test\",\"semver\":\"test\",\"origin_url\":\"test\",\"timestamp\":\"test\"}";
    let build_information = BuildInformation {
        commit_sha: "test".to_string(),
        commit_timestamp: "test".to_string(),
        semver: "test".to_string(),
        origin_url: "test".to_string(),
        timestamp: "test".to_string()
    };
    let build_information_json = serde_json::from_str::<BuildInformation>(json).unwrap();
    assert_eq!(build_information_json, build_information);
}

#[test]
fn test_stoat_config_deserialize() {
    let stoat_config = StoatConfig {
        revolt: "test".to_string(),
        features: StoatFeatures {
            captcha: CaptchaFeature {
                enabled: true,
                key: "test".to_string(),
            },
            email: false,
            invite_only: false,
            autumn: Feature {
                enabled: true,
                url: "test".to_string(),
            },
            january: Feature {
                enabled: true,
                url: "test".to_string(),
            },
            livekit: VoiceFeature {
                enabled: true,
                nodes: vec![]
            }
        },
        ws: "test".to_string(),
        app: "test".to_string(),
        vapid: "test".to_string(),
        build: BuildInformation {
            commit_sha: "test".to_string(),
            commit_timestamp: "test".to_string(),
            semver: "test".to_string(),
            origin_url: "test".to_string(),
            timestamp: "test".to_string()
        }
    };
    let json = serde_json::to_string(&stoat_config).unwrap();
    assert_eq!("{\"revolt\":\"test\",\"features\":{\"captcha\":{\"enabled\":true,\"key\":\"test\"},\"email\":false,\"invite_only\":false,\"autumn\":{\"enabled\":true,\"url\":\"test\"},\"january\":{\"enabled\":true,\"url\":\"test\"},\"livekit\":{\"enabled\":true,\"nodes\":[]}},\"ws\":\"test\",\"app\":\"test\",\"vapid\":\"test\",\"build\":{\"commit_sha\":\"test\",\"commit_timestamp\":\"test\",\"semver\":\"test\",\"origin_url\":\"test\",\"timestamp\":\"test\"}}", json);
}

#[test]
fn test_stoat_config_serialize() {
    let stoat_config = StoatConfig {
        revolt: "test".to_string(),
        features: StoatFeatures {
            captcha: CaptchaFeature {
                enabled: true,
                key: "test".to_string(),
            },
            email: false,
            invite_only: false,
            autumn: Feature {
                enabled: true,
                url: "test".to_string(),
            },
            january: Feature {
                enabled: true,
                url: "test".to_string(),
            },
            livekit: VoiceFeature {
                enabled: true,
                nodes: vec![]
            }
        },
        ws: "test".to_string(),
        app: "test".to_string(),
        vapid: "test".to_string(),
        build: BuildInformation {
            commit_sha: "test".to_string(),
            commit_timestamp: "test".to_string(),
            semver: "test".to_string(),
            origin_url: "test".to_string(),
            timestamp: "test".to_string()
        }
    };
    let json = "{\"revolt\":\"test\",\"features\":{\"captcha\":{\"enabled\":true,\"key\":\"test\"},\"email\":false,\"invite_only\":false,\"autumn\":{\"enabled\":true,\"url\":\"test\"},\"january\":{\"enabled\":true,\"url\":\"test\"},\"livekit\":{\"enabled\":true,\"nodes\":[]}},\"ws\":\"test\",\"app\":\"test\",\"vapid\":\"test\",\"build\":{\"commit_sha\":\"test\",\"commit_timestamp\":\"test\",\"semver\":\"test\",\"origin_url\":\"test\",\"timestamp\":\"test\"}}";
    let stoat_config_json = serde_json::from_str::<StoatConfig>(json).unwrap();
    assert_eq!(stoat_config, stoat_config_json);
}

#[test]
fn test_autumn_response_deserialize() {
    let autumn_response = AutumnResponse {
        id: "test".to_string()
    };
    let json = serde_json::to_string(&autumn_response).unwrap();
    assert_eq!("{\"id\":\"test\"}", json);
}

#[test]
fn test_autumn_response_serialize() {
    let autumn_response = AutumnResponse {
        id: "test".to_string()
    };
    let json = "{\"id\":\"test\"}";
    let autumn_response_json = serde_json::from_str::<AutumnResponse>(json).unwrap();
    assert_eq!(autumn_response, autumn_response_json);
}

#[test]
fn test_ratelimit_failure_deserialize() {
    let ratelimit_failure = RatelimitFailure {
        retry_after: 100
    };
    let json = serde_json::to_string(&ratelimit_failure).unwrap();
    assert_eq!("{\"retry_after\":100}", json);
}

#[test]
fn test_ratelimit_failure_serialize() {
    let ratelimit_failure = RatelimitFailure {
        retry_after: 100
    };
    let json = "{\"retry_after\":100}";
    let ratelimit_failure_json = serde_json::from_str::<RatelimitFailure>(json).unwrap();
    assert_eq!(ratelimit_failure, ratelimit_failure_json);
}

#[test]
fn test_tag_deserialize() {
    let tag = Tag::Attachments;
    let json = serde_json::to_string(&tag).unwrap();
    assert_eq!("\"Attachments\"", json);

    let tag = Tag::Avatars;
    let json = serde_json::to_string(&tag).unwrap();
    assert_eq!("\"Avatars\"", json);

    let tag = Tag::Backgrounds;
    let json = serde_json::to_string(&tag).unwrap();
    assert_eq!("\"Backgrounds\"", json);

    let tag = Tag::Icons;
    let json = serde_json::to_string(&tag).unwrap();
    assert_eq!("\"Icons\"", json);

    let tag = Tag::Banners;
    let json = serde_json::to_string(&tag).unwrap();
    assert_eq!("\"Banners\"", json);

    let tag = Tag::Emojis;
    let json = serde_json::to_string(&tag).unwrap();
    assert_eq!("\"Emojis\"", json);
}

#[test]
fn test_tag_serialize() {
    let json = "\"Attachments\"";
    let tag = serde_json::from_str::<Tag>(json).unwrap();
    assert_eq!(tag, Tag::Attachments);

    let json = "\"Avatars\"";
    let tag = serde_json::from_str::<Tag>(json).unwrap();
    assert_eq!(tag, Tag::Avatars);

    let json = "\"Backgrounds\"";
    let tag = serde_json::from_str::<Tag>(json).unwrap();
    assert_eq!(tag, Tag::Backgrounds);

    let json = "\"Icons\"";
    let tag = serde_json::from_str::<Tag>(json).unwrap();
    assert_eq!(tag, Tag::Icons);

    let json = "\"Banners\"";
    let tag = serde_json::from_str::<Tag>(json).unwrap();
    assert_eq!(tag, Tag::Banners);

    let json = "\"Emojis\"";
    let tag = serde_json::from_str::<Tag>(json).unwrap();
    assert_eq!(tag, Tag::Emojis);
}

#[test]
fn test_tag_try_from() {
    let attachments = Tag::try_from("attachments");
    if let Ok(attachments) = attachments {
        assert_eq!(attachments, Tag::Attachments);
    } else {
        panic!("Tag::try_from(\"attachments\") failed");
    }
    let avatars = Tag::try_from("avatars");
    if let Ok(avatars) = avatars {
        assert_eq!(avatars, Tag::Avatars);
    } else {
        panic!("Tag::try_from(\"avatars\") failed");
    }
    let backgrounds = Tag::try_from("backgrounds");
    if let Ok(backgrounds) = backgrounds {
        assert_eq!(backgrounds, Tag::Backgrounds);
    } else {
        panic!("Tag::try_from(\"backgrounds\") failed");
    }
    let icons = Tag::try_from("icons");
    if let Ok(icons) = icons {
        assert_eq!(icons, Tag::Icons);
    } else {
        panic!("Tag::try_from(\"icons\") failed");
    }
    let banners = Tag::try_from("banners");
    if let Ok(banners) = banners {
        assert_eq!(banners, Tag::Banners);
    } else {
        panic!("Tag::try_from(\"banners\") failed");
    }
    let emojis = Tag::try_from("emojis");
    if let Ok(emojis) = emojis {
        assert_eq!(emojis, Tag::Emojis);
    } else {
        panic!("Tag::try_from(\"emojis\") failed");
    }
    let invalid = Tag::try_from("invalid");
    if let Err(stoat::Error::InvalidTag) = invalid {
        assert!(true)
    } else {
        panic!("Expected InvalidTag error");
    }
}