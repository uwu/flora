use deno_error::JsErrorBox;
use serde::Deserialize;
use serde_json::Value;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateComponent, CreateContainer, CreateContainerComponent,
    CreateFile, CreateFileUpload, CreateInputText, CreateLabel, CreateMediaGallery,
    CreateMediaGalleryItem, CreateSection, CreateSectionAccessory, CreateSectionComponent,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, CreateSeparator,
    CreateTextDisplay, CreateThumbnail, CreateUnfurledMediaItem, Spacing,
};
use serenity::model::Colour;
use serenity::model::application::{ButtonStyle, InputTextStyle};
use serenity::model::channel::ChannelType;
use serenity::model::id::{GenericChannelId, RoleId, SkuId, UserId};
use serenity::model::prelude::ReactionType;

pub fn parse_components(raw: Vec<Value>) -> Result<Vec<CreateComponent<'static>>, JsErrorBox> {
    raw.into_iter().map(parse_component).collect()
}

fn parse_component(value: Value) -> Result<CreateComponent<'static>, JsErrorBox> {
    let kind = component_kind(&value)?;
    match kind {
        1 => {
            let row: RawActionRow = serde_json::from_value(value)
                .map_err(|err| JsErrorBox::generic(err.to_string()))?;
            let mut buttons = Vec::new();
            let mut select_menus = Vec::new();
            for child in row.components {
                match component_kind(&child)? {
                    2 => buttons.push(parse_button(child)?),
                    3 | 5 | 6 | 7 | 8 => select_menus.push(parse_select_menu(child)?),
                    other => {
                        return Err(JsErrorBox::generic(format!(
                            "Unsupported action row component type {other}"
                        )));
                    }
                }
            }
            if !select_menus.is_empty() {
                if select_menus.len() > 1 {
                    return Err(JsErrorBox::generic(
                        "Action row supports only one select menu",
                    ));
                }
                Ok(CreateComponent::ActionRow(CreateActionRow::SelectMenu(
                    select_menus.remove(0),
                )))
            } else {
                Ok(CreateComponent::ActionRow(CreateActionRow::Buttons(
                    buttons.into(),
                )))
            }
        }
        2 => Ok(CreateComponent::ActionRow(CreateActionRow::Buttons(
            vec![parse_button(value)?].into(),
        ))),
        3 | 5 | 6 | 7 | 8 => Ok(CreateComponent::ActionRow(CreateActionRow::SelectMenu(
            parse_select_menu(value)?,
        ))),
        9 => Ok(CreateComponent::Section(parse_section(value)?)),
        10 => Ok(CreateComponent::TextDisplay(parse_text_display(value)?)),
        11 => Err(JsErrorBox::generic(
            "Thumbnail components are only valid as section accessories",
        )),
        12 => Ok(CreateComponent::MediaGallery(parse_media_gallery(value)?)),
        13 => Ok(CreateComponent::File(parse_file(value)?)),
        14 => Ok(CreateComponent::Separator(parse_separator(value)?)),
        17 => Ok(CreateComponent::Container(parse_container(value)?)),
        18 => Ok(CreateComponent::Label(parse_label(value)?)),
        other => Err(JsErrorBox::generic(format!(
            "Unsupported component type {other}"
        ))),
    }
}

fn component_kind(value: &Value) -> Result<u8, JsErrorBox> {
    value
        .get("type")
        .and_then(|v| v.as_u64())
        .map(|v| v as u8)
        .ok_or_else(|| JsErrorBox::generic("Component missing numeric type"))
}

#[derive(Deserialize)]
struct RawActionRow {
    components: Vec<Value>,
}

#[derive(Deserialize)]
struct RawButton {
    #[serde(default)]
    style: Option<u8>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    emoji: Option<Value>,
    #[serde(default)]
    custom_id: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    disabled: Option<bool>,
    #[serde(default)]
    sku_id: Option<String>,
}

fn parse_button(value: Value) -> Result<CreateButton<'static>, JsErrorBox> {
    let raw: RawButton =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let style = raw.style.unwrap_or(1);
    let mut button = match style {
        5 => {
            let url = raw
                .url
                .ok_or_else(|| JsErrorBox::generic("Link button requires url"))?;
            CreateButton::new_link(url)
        }
        6 => {
            let sku_id = raw
                .sku_id
                .ok_or_else(|| JsErrorBox::generic("Premium button requires sku_id"))?;
            let sku_id = sku_id
                .parse::<u64>()
                .map_err(|_| JsErrorBox::generic("Invalid sku_id"))?;
            CreateButton::new_premium(SkuId::new(sku_id))
        }
        _ => {
            let custom_id = raw
                .custom_id
                .ok_or_else(|| JsErrorBox::generic("Button requires custom_id"))?;
            let mut btn = CreateButton::new(custom_id);
            btn = btn.style(match style {
                1 => ButtonStyle::Primary,
                2 => ButtonStyle::Secondary,
                3 => ButtonStyle::Success,
                4 => ButtonStyle::Danger,
                other => ButtonStyle::Unknown(other),
            });
            btn
        }
    };

    if let Some(label) = raw.label {
        button = button.label(label);
    }
    if let Some(emoji) = raw.emoji {
        button = button.emoji(parse_reaction_type(&emoji)?);
    }
    if let Some(disabled) = raw.disabled {
        button = button.disabled(disabled);
    }
    Ok(button)
}

#[derive(Deserialize)]
struct RawSelectMenu {
    custom_id: String,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(default)]
    min_values: Option<u8>,
    #[serde(default)]
    max_values: Option<u8>,
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    disabled: Option<bool>,
    #[serde(default)]
    options: Option<Vec<RawSelectOption>>,
    #[serde(default)]
    channel_types: Option<Vec<ChannelType>>,
    #[serde(default)]
    default_values: Option<Vec<RawSelectDefault>>,
    #[serde(default)]
    default_users: Option<Vec<String>>,
    #[serde(default)]
    default_roles: Option<Vec<String>>,
    #[serde(default)]
    default_channels: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawSelectDefault {
    id: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct RawSelectOption {
    label: String,
    value: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    emoji: Option<Value>,
    #[serde(default)]
    default: Option<bool>,
}

fn parse_select_menu(value: Value) -> Result<CreateSelectMenu<'static>, JsErrorBox> {
    let kind = component_kind(&value)?;
    let raw: RawSelectMenu =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;

    let mut default_users: Vec<UserId> = Vec::new();
    let mut default_roles: Vec<RoleId> = Vec::new();
    let mut default_channels: Vec<GenericChannelId> = Vec::new();

    if let Some(values) = raw.default_values {
        for value in values {
            match value.kind.as_str() {
                "user" => {
                    if let Ok(id) = value.id.parse::<u64>() {
                        default_users.push(UserId::new(id));
                    }
                }
                "role" => {
                    if let Ok(id) = value.id.parse::<u64>() {
                        default_roles.push(RoleId::new(id));
                    }
                }
                "channel" => {
                    if let Ok(id) = value.id.parse::<u64>() {
                        default_channels.push(GenericChannelId::new(id));
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(values) = raw.default_users {
        for value in values {
            if let Ok(id) = value.parse::<u64>() {
                default_users.push(UserId::new(id));
            }
        }
    }

    if let Some(values) = raw.default_roles {
        for value in values {
            if let Ok(id) = value.parse::<u64>() {
                default_roles.push(RoleId::new(id));
            }
        }
    }

    if let Some(values) = raw.default_channels {
        for value in values {
            if let Ok(id) = value.parse::<u64>() {
                default_channels.push(GenericChannelId::new(id));
            }
        }
    }

    let menu_kind = match kind {
        3 => {
            let options = raw
                .options
                .ok_or_else(|| JsErrorBox::generic("String select requires options"))?;
            let built_options = options
                .into_iter()
                .map(|opt| {
                    let mut option = CreateSelectMenuOption::new(opt.label, opt.value);
                    if let Some(description) = opt.description {
                        option = option.description(description);
                    }
                    if let Some(emoji) = opt.emoji {
                        option = option.emoji(parse_reaction_type(&emoji)?);
                    }
                    if let Some(default) = opt.default {
                        option = option.default_selection(default);
                    }
                    Ok(option)
                })
                .collect::<Result<Vec<_>, JsErrorBox>>()?;
            CreateSelectMenuKind::String {
                options: built_options.into(),
            }
        }
        5 => CreateSelectMenuKind::User {
            default_users: if default_users.is_empty() {
                None
            } else {
                Some(default_users.into())
            },
        },
        6 => CreateSelectMenuKind::Role {
            default_roles: if default_roles.is_empty() {
                None
            } else {
                Some(default_roles.into())
            },
        },
        7 => CreateSelectMenuKind::Mentionable {
            default_users: if default_users.is_empty() {
                None
            } else {
                Some(default_users.into())
            },
            default_roles: if default_roles.is_empty() {
                None
            } else {
                Some(default_roles.into())
            },
        },
        8 => CreateSelectMenuKind::Channel {
            channel_types: raw.channel_types.map(Into::into),
            default_channels: if default_channels.is_empty() {
                None
            } else {
                Some(default_channels.into())
            },
        },
        other => {
            return Err(JsErrorBox::generic(format!(
                "Unsupported select menu type {other}"
            )));
        }
    };

    let mut menu = CreateSelectMenu::new(raw.custom_id, menu_kind);
    if let Some(placeholder) = raw.placeholder {
        menu = menu.placeholder(placeholder);
    }
    if let Some(min) = raw.min_values {
        menu = menu.min_values(min);
    }
    if let Some(max) = raw.max_values {
        menu = menu.max_values(max);
    }
    if let Some(required) = raw.required {
        menu = menu.required(required);
    }
    if let Some(disabled) = raw.disabled {
        menu = menu.disabled(disabled);
    }
    Ok(menu)
}

#[derive(Deserialize)]
struct RawTextDisplay {
    content: String,
}

fn parse_text_display(value: Value) -> Result<CreateTextDisplay<'static>, JsErrorBox> {
    let raw: RawTextDisplay =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(CreateTextDisplay::new(raw.content))
}

#[derive(Deserialize)]
struct RawSection {
    components: Vec<Value>,
    accessory: Value,
}

fn parse_section(value: Value) -> Result<CreateSection<'static>, JsErrorBox> {
    let raw: RawSection =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let components = raw
        .components
        .into_iter()
        .map(parse_text_display)
        .map(|res| res.map(CreateSectionComponent::TextDisplay))
        .collect::<Result<Vec<_>, JsErrorBox>>()?;
    let accessory = parse_section_accessory(raw.accessory)?;
    Ok(CreateSection::new(components, accessory))
}

fn parse_section_accessory(value: Value) -> Result<CreateSectionAccessory<'static>, JsErrorBox> {
    match component_kind(&value)? {
        2 => Ok(CreateSectionAccessory::Button(parse_button(value)?)),
        11 => Ok(CreateSectionAccessory::Thumbnail(parse_thumbnail(value)?)),
        other => Err(JsErrorBox::generic(format!(
            "Unsupported section accessory type {other}"
        ))),
    }
}

#[derive(Deserialize)]
struct RawThumbnail {
    media: RawMediaItem,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    spoiler: Option<bool>,
}

#[derive(Deserialize)]
struct RawMediaItem {
    url: String,
}

fn parse_media_item(raw: RawMediaItem) -> CreateUnfurledMediaItem<'static> {
    CreateUnfurledMediaItem::new(raw.url)
}

fn parse_thumbnail(value: Value) -> Result<CreateThumbnail<'static>, JsErrorBox> {
    let raw: RawThumbnail =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let mut thumb = CreateThumbnail::new(parse_media_item(raw.media));
    if let Some(description) = raw.description {
        thumb = thumb.description(description);
    }
    if let Some(spoiler) = raw.spoiler {
        thumb = thumb.spoiler(spoiler);
    }
    Ok(thumb)
}

#[derive(Deserialize)]
struct RawMediaGallery {
    items: Vec<RawMediaGalleryItem>,
}

#[derive(Deserialize)]
struct RawMediaGalleryItem {
    media: RawMediaItem,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    spoiler: Option<bool>,
}

fn parse_media_gallery(value: Value) -> Result<CreateMediaGallery<'static>, JsErrorBox> {
    let raw: RawMediaGallery =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let items = raw
        .items
        .into_iter()
        .map(|item| {
            let mut built = CreateMediaGalleryItem::new(parse_media_item(item.media));
            if let Some(description) = item.description {
                built = built.description(description);
            }
            if let Some(spoiler) = item.spoiler {
                built = built.spoiler(spoiler);
            }
            built
        })
        .collect::<Vec<_>>();
    Ok(CreateMediaGallery::new(items))
}

#[derive(Deserialize)]
struct RawFile {
    file: RawMediaItem,
    #[serde(default)]
    spoiler: Option<bool>,
}

fn parse_file(value: Value) -> Result<CreateFile<'static>, JsErrorBox> {
    let raw: RawFile =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let mut file = CreateFile::new(parse_media_item(raw.file));
    if let Some(spoiler) = raw.spoiler {
        file = file.spoiler(spoiler);
    }
    Ok(file)
}

#[derive(Deserialize)]
struct RawSeparator {
    divider: bool,
    #[serde(default)]
    spacing: Option<RawSpacing>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawSpacing {
    Number(u8),
    Text(String),
}

fn parse_separator(value: Value) -> Result<CreateSeparator, JsErrorBox> {
    let raw: RawSeparator =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let mut separator = CreateSeparator::new(raw.divider);
    if let Some(spacing) = raw.spacing {
        let spacing = match spacing {
            RawSpacing::Number(1) => Spacing::Small,
            RawSpacing::Number(2) => Spacing::Large,
            RawSpacing::Number(other) => Spacing::Unknown(other),
            RawSpacing::Text(text) => match text.as_str() {
                "small" => Spacing::Small,
                "large" => Spacing::Large,
                _ => return Err(JsErrorBox::generic("Invalid separator spacing")),
            },
        };
        separator = separator.spacing(spacing);
    }
    Ok(separator)
}

#[derive(Deserialize)]
struct RawContainer {
    components: Vec<Value>,
    #[serde(default)]
    accent_color: Option<u32>,
    #[serde(default)]
    spoiler: Option<bool>,
}

fn parse_container(value: Value) -> Result<CreateContainer<'static>, JsErrorBox> {
    let raw: RawContainer =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let components = raw
        .components
        .into_iter()
        .map(parse_container_component)
        .collect::<Result<Vec<_>, JsErrorBox>>()?;
    let mut container = CreateContainer::new(components);
    if let Some(color) = raw.accent_color {
        container = container.accent_color(Colour::new(color));
    }
    if let Some(spoiler) = raw.spoiler {
        container = container.spoiler(spoiler);
    }
    Ok(container)
}

fn parse_container_component(
    value: Value,
) -> Result<CreateContainerComponent<'static>, JsErrorBox> {
    match component_kind(&value)? {
        1 => {
            let row: RawActionRow = serde_json::from_value(value)
                .map_err(|err| JsErrorBox::generic(err.to_string()))?;
            let mut buttons = Vec::new();
            let mut select_menus = Vec::new();
            for child in row.components {
                match component_kind(&child)? {
                    2 => buttons.push(parse_button(child)?),
                    3 | 5 | 6 | 7 | 8 => select_menus.push(parse_select_menu(child)?),
                    other => {
                        return Err(JsErrorBox::generic(format!(
                            "Unsupported container action row component type {other}"
                        )));
                    }
                }
            }
            if !select_menus.is_empty() {
                if select_menus.len() > 1 {
                    return Err(JsErrorBox::generic(
                        "Action row supports only one select menu",
                    ));
                }
                Ok(CreateContainerComponent::ActionRow(
                    CreateActionRow::SelectMenu(select_menus.remove(0)),
                ))
            } else {
                Ok(CreateContainerComponent::ActionRow(
                    CreateActionRow::Buttons(buttons.into()),
                ))
            }
        }
        9 => Ok(CreateContainerComponent::Section(parse_section(value)?)),
        10 => Ok(CreateContainerComponent::TextDisplay(parse_text_display(
            value,
        )?)),
        12 => Ok(CreateContainerComponent::MediaGallery(parse_media_gallery(
            value,
        )?)),
        13 => Ok(CreateContainerComponent::File(parse_file(value)?)),
        14 => Ok(CreateContainerComponent::Separator(parse_separator(value)?)),
        other => Err(JsErrorBox::generic(format!(
            "Unsupported container component type {other}"
        ))),
    }
}

#[derive(Deserialize)]
struct RawLabel {
    label: String,
    #[serde(default)]
    description: Option<String>,
    component: Value,
}

fn parse_label(value: Value) -> Result<CreateLabel<'static>, JsErrorBox> {
    let raw: RawLabel =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let mut label = match component_kind(&raw.component)? {
        3 | 5 | 6 | 7 | 8 => CreateLabel::select_menu(raw.label, parse_select_menu(raw.component)?),
        4 => CreateLabel::input_text(raw.label, parse_input_text(raw.component)?),
        19 => CreateLabel::file_upload(raw.label, parse_file_upload(raw.component)?),
        other => {
            return Err(JsErrorBox::generic(format!(
                "Unsupported label component type {other}"
            )));
        }
    };
    if let Some(description) = raw.description {
        label = label.description(description);
    }
    Ok(label)
}

#[derive(Deserialize)]
struct RawInputText {
    custom_id: String,
    #[serde(default)]
    style: Option<u8>,
    #[serde(default)]
    min_length: Option<u16>,
    #[serde(default)]
    max_length: Option<u16>,
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    placeholder: Option<String>,
}

fn parse_input_text(value: Value) -> Result<CreateInputText<'static>, JsErrorBox> {
    let raw: RawInputText =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let style = match raw.style.unwrap_or(1) {
        1 => InputTextStyle::Short,
        2 => InputTextStyle::Paragraph,
        other => InputTextStyle::Unknown(other),
    };
    let mut input = CreateInputText::new(style, raw.custom_id);
    if let Some(min) = raw.min_length {
        input = input.min_length(min);
    }
    if let Some(max) = raw.max_length {
        input = input.max_length(max);
    }
    if let Some(required) = raw.required {
        input = input.required(required);
    }
    if let Some(value) = raw.value {
        input = input.value(value);
    }
    if let Some(placeholder) = raw.placeholder {
        input = input.placeholder(placeholder);
    }
    Ok(input)
}

#[derive(Deserialize)]
struct RawFileUpload {
    custom_id: String,
    #[serde(default)]
    min_values: Option<u8>,
    #[serde(default)]
    max_values: Option<u8>,
    #[serde(default)]
    required: Option<bool>,
}

fn parse_file_upload(value: Value) -> Result<CreateFileUpload<'static>, JsErrorBox> {
    let raw: RawFileUpload =
        serde_json::from_value(value).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let mut upload = CreateFileUpload::new(raw.custom_id);
    if let Some(min) = raw.min_values {
        upload = upload.min_values(min);
    }
    if let Some(max) = raw.max_values {
        upload = upload.max_values(max);
    }
    if let Some(required) = raw.required {
        upload = upload.required(required);
    }
    Ok(upload)
}

fn parse_reaction_type(value: &Value) -> Result<ReactionType, JsErrorBox> {
    if let Some(text) = value.as_str() {
        return ReactionType::try_from(text)
            .map_err(|_| JsErrorBox::generic("Invalid emoji string"));
    }
    serde_json::from_value(value.clone()).map_err(|err| JsErrorBox::generic(err.to_string()))
}
