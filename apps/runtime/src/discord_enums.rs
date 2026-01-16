use t0x::__private::{AstBuilder, NumberBase, OxcVec, SPAN, TSType};
use t0x::T0x;

/// Message flags used for Discord messages.
pub struct MessageFlags;

impl MessageFlags {
    /// This message has been published to subscribed channels (via Channel Following).
    pub const CROSSPOSTED: u32 = 1 << 0;
    /// This message originated from a message in another channel (via Channel Following).
    pub const IS_CROSSPOST: u32 = 1 << 1;
    /// Do not include any embeds when serializing this message.
    pub const SUPPRESS_EMBEDS: u32 = 1 << 2;
    /// The source message for this crosspost has been deleted (via Channel Following).
    pub const SOURCE_MESSAGE_DELETED: u32 = 1 << 3;
    /// This message came from the urgent message system.
    pub const URGENT: u32 = 1 << 4;
    /// This message has an associated thread, with the same id as the message.
    pub const HAS_THREAD: u32 = 1 << 5;
    /// This message is only visible to the user who invoked the Interaction.
    pub const EPHEMERAL: u32 = 1 << 6;
    /// This message is an Interaction Response and the bot is "thinking".
    pub const LOADING: u32 = 1 << 7;
    /// This message failed to mention some roles and add their members to the thread.
    pub const FAILED_TO_MENTION_SOME_ROLES_IN_THREAD: u32 = 1 << 8;
    /// This message will not trigger push and desktop notifications.
    pub const SUPPRESS_NOTIFICATIONS: u32 = 1 << 12;
    /// This message is a voice message.
    pub const IS_VOICE_MESSAGE: u32 = 1 << 13;
    /// Enables support for sending Components V2.
    pub const IS_COMPONENTS_V2: u32 = 1 << 15;
}

/// Discord component types.
pub struct ComponentType;

impl ComponentType {
    /// A component action row (V1).
    pub const ACTION_ROW: u32 = 1;
    /// A button component.
    pub const BUTTON: u32 = 2;
    /// A string select menu component.
    pub const STRING_SELECT: u32 = 3;
    /// A text input component for modals.
    pub const INPUT_TEXT: u32 = 4;
    /// A user select menu component.
    pub const USER_SELECT: u32 = 5;
    /// A role select menu component.
    pub const ROLE_SELECT: u32 = 6;
    /// A mentionable select menu component.
    pub const MENTIONABLE_SELECT: u32 = 7;
    /// A channel select menu component.
    pub const CHANNEL_SELECT: u32 = 8;
    /// A section component (V2).
    pub const SECTION: u32 = 9;
    /// A text display component (V2).
    pub const TEXT_DISPLAY: u32 = 10;
    /// A thumbnail component (V2).
    pub const THUMBNAIL: u32 = 11;
    /// A media gallery component (V2).
    pub const MEDIA_GALLERY: u32 = 12;
    /// A file component (V2).
    pub const FILE: u32 = 13;
    /// A separator component (V2).
    pub const SEPARATOR: u32 = 14;
    /// A container component (V2).
    pub const CONTAINER: u32 = 17;
    /// A label component (V2).
    pub const LABEL: u32 = 18;
    /// A file upload component (V2).
    pub const FILE_UPLOAD: u32 = 19;
}

/// Button styles for message components.
pub struct ButtonStyle;

impl ButtonStyle {
    /// Primary button style.
    pub const PRIMARY: u32 = 1;
    /// Secondary button style.
    pub const SECONDARY: u32 = 2;
    /// Success button style.
    pub const SUCCESS: u32 = 3;
    /// Danger button style.
    pub const DANGER: u32 = 4;
}

/// Input text styles for modal components.
pub struct InputTextStyle;

impl InputTextStyle {
    /// Short input text.
    pub const SHORT: u32 = 1;
    /// Paragraph input text.
    pub const PARAGRAPH: u32 = 2;
}

impl T0x for MessageFlags {
    const NAME: &'static str = "MessageFlags";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        TSType::TSNumberKeyword(ast.alloc_ts_number_keyword(SPAN))
    }

    fn value_def() -> Option<String> {
        Some(enum_value_def("MessageFlags", message_flags_entries()))
    }
}

impl T0x for ComponentType {
    const NAME: &'static str = "ComponentType";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        number_union(ast, component_type_entries())
    }

    fn value_def() -> Option<String> {
        Some(enum_value_def("ComponentType", component_type_entries()))
    }
}

impl T0x for ButtonStyle {
    const NAME: &'static str = "ButtonStyle";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        number_union(ast, button_style_entries())
    }

    fn value_def() -> Option<String> {
        Some(enum_value_def("ButtonStyle", button_style_entries()))
    }
}

impl T0x for InputTextStyle {
    const NAME: &'static str = "InputTextStyle";

    fn ts_type<'a>(ast: AstBuilder<'a>) -> TSType<'a> {
        number_union(ast, input_text_style_entries())
    }

    fn value_def() -> Option<String> {
        Some(enum_value_def("InputTextStyle", input_text_style_entries()))
    }
}

fn message_flags_entries() -> &'static [(&'static str, u32)] {
    const ENTRIES: [(&str, u32); 12] = [
        ("CROSSPOSTED", MessageFlags::CROSSPOSTED),
        ("IS_CROSSPOST", MessageFlags::IS_CROSSPOST),
        ("SUPPRESS_EMBEDS", MessageFlags::SUPPRESS_EMBEDS),
        (
            "SOURCE_MESSAGE_DELETED",
            MessageFlags::SOURCE_MESSAGE_DELETED,
        ),
        ("URGENT", MessageFlags::URGENT),
        ("HAS_THREAD", MessageFlags::HAS_THREAD),
        ("EPHEMERAL", MessageFlags::EPHEMERAL),
        ("LOADING", MessageFlags::LOADING),
        (
            "FAILED_TO_MENTION_SOME_ROLES_IN_THREAD",
            MessageFlags::FAILED_TO_MENTION_SOME_ROLES_IN_THREAD,
        ),
        (
            "SUPPRESS_NOTIFICATIONS",
            MessageFlags::SUPPRESS_NOTIFICATIONS,
        ),
        ("IS_VOICE_MESSAGE", MessageFlags::IS_VOICE_MESSAGE),
        ("IS_COMPONENTS_V2", MessageFlags::IS_COMPONENTS_V2),
    ];
    &ENTRIES
}

fn component_type_entries() -> &'static [(&'static str, u32)] {
    const ENTRIES: [(&str, u32); 17] = [
        ("ActionRow", ComponentType::ACTION_ROW),
        ("Button", ComponentType::BUTTON),
        ("StringSelect", ComponentType::STRING_SELECT),
        ("InputText", ComponentType::INPUT_TEXT),
        ("UserSelect", ComponentType::USER_SELECT),
        ("RoleSelect", ComponentType::ROLE_SELECT),
        ("MentionableSelect", ComponentType::MENTIONABLE_SELECT),
        ("ChannelSelect", ComponentType::CHANNEL_SELECT),
        ("Section", ComponentType::SECTION),
        ("TextDisplay", ComponentType::TEXT_DISPLAY),
        ("Thumbnail", ComponentType::THUMBNAIL),
        ("MediaGallery", ComponentType::MEDIA_GALLERY),
        ("File", ComponentType::FILE),
        ("Separator", ComponentType::SEPARATOR),
        ("Container", ComponentType::CONTAINER),
        ("Label", ComponentType::LABEL),
        ("FileUpload", ComponentType::FILE_UPLOAD),
    ];
    &ENTRIES
}

fn button_style_entries() -> &'static [(&'static str, u32)] {
    const ENTRIES: [(&str, u32); 4] = [
        ("Primary", ButtonStyle::PRIMARY),
        ("Secondary", ButtonStyle::SECONDARY),
        ("Success", ButtonStyle::SUCCESS),
        ("Danger", ButtonStyle::DANGER),
    ];
    &ENTRIES
}

fn input_text_style_entries() -> &'static [(&'static str, u32)] {
    const ENTRIES: [(&str, u32); 2] = [
        ("Short", InputTextStyle::SHORT),
        ("Paragraph", InputTextStyle::PARAGRAPH),
    ];
    &ENTRIES
}

fn number_union<'a>(ast: AstBuilder<'a>, entries: &[(&str, u32)]) -> TSType<'a> {
    let mut types = OxcVec::new_in(ast.allocator);
    for (_, value) in entries {
        let literal =
            ast.ts_literal_numeric_literal(SPAN, f64::from(*value), None, NumberBase::Decimal);
        let literal_type = ast.alloc_ts_literal_type(SPAN, literal);
        types.push(TSType::TSLiteralType(literal_type));
    }
    TSType::TSUnionType(ast.alloc_ts_union_type(SPAN, types))
}

fn enum_value_def(name: &str, entries: &[(&str, u32)]) -> String {
    let mut output = String::new();
    output.push_str("export const ");
    output.push_str(name);
    output.push_str(" = {\n");
    for (key, value) in entries {
        output.push_str("  ");
        output.push_str(key);
        output.push_str(": ");
        output.push_str(&value.to_string());
        output.push_str(",\n");
    }
    output.push_str("} as const;\n");
    output
}
