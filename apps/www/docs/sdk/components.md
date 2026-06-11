---
title: 'Components'
description: 'Create buttons, select menus, and interactive messages'
outline: deep
---

# Components

Discord components let users interact with your bot through buttons, select menus, inputs, and richer components v2 layouts. flora gives you small builder functions for creating those payloads.

## Action Rows

Buttons and select menus are usually placed in action rows. A message can have up to 5 action rows.

```typescript
const row = actionRow().addComponents(
  button().setCustomId('yes').setLabel('Yes').setStyle(ButtonStyle.Success),
  button().setCustomId('no').setLabel('No').setStyle(ButtonStyle.Danger)
)

await ctx.reply({
  content: 'Do you agree?',
  components: [row.toJSON()]
})
```

## Buttons

### Basic Button

```typescript
const btn = button().setCustomId('click_me').setLabel('Click me').setStyle(ButtonStyle.Primary)
```

### Button Styles

| Style                   | Example                                |
| ----------------------- | -------------------------------------- |
| `ButtonStyle.Primary`   | Main action.                           |
| `ButtonStyle.Secondary` | Neutral action.                        |
| `ButtonStyle.Success`   | Confirm or positive action.            |
| `ButtonStyle.Danger`    | Delete, cancel, or destructive action. |

```typescript
const row = actionRow().addComponents(
  button().setCustomId('primary').setLabel('Primary').setStyle(ButtonStyle.Primary),
  button().setCustomId('secondary').setLabel('Secondary').setStyle(ButtonStyle.Secondary),
  button().setCustomId('success').setLabel('Success').setStyle(ButtonStyle.Success),
  button().setCustomId('danger').setLabel('Danger').setStyle(ButtonStyle.Danger)
)
```

### Link Button

Buttons that open URLs do not need custom IDs. `setUrl()` configures the link style for you.

```typescript
const docs = button().setUrl('https://flora.dev').setLabel('Documentation')
```

### Button with Emoji

```typescript
const like = button()
  .setCustomId('like')
  .setLabel('Like')
  .setEmoji({ name: 'thumbs_up' })
  .setStyle(ButtonStyle.Primary)
```

### Disabled Button

```typescript
const disabled = button()
  .setCustomId('disabled')
  .setLabel('Unavailable')
  .setDisabled(true)
  .setStyle(ButtonStyle.Secondary)
```

## Select Menus

### String Select

Use `stringSelect()` when users should pick from options you define.

```typescript
const select = stringSelect('role_select')
  .setPlaceholder('Choose a role')
  .addOptions(
    { label: 'Developer', value: 'dev' },
    { label: 'Designer', value: 'design' },
    { label: 'Manager', value: 'manager' }
  )

const row = actionRow().addComponents(select)

await ctx.reply({
  content: 'Select your role:',
  components: [row.toJSON()]
})
```

### User Select

```typescript
const select = userSelect('user_picker')
  .setPlaceholder('Select a user')
  .setMinValues(1)
  .setMaxValues(5)
```

### Role Select

```typescript
const select = roleSelect('role_picker').setPlaceholder('Select roles').setMaxValues(3)
```

### Channel Select

```typescript
const select = channelSelect('channel_picker').setPlaceholder('Select a channel')
```

### Mentionable Select

```typescript
const select = mentionableSelect('mention_picker').setPlaceholder('Select users or roles')
```

## Input Text

Text inputs are used in modal-style flows.

```typescript
const input = inputText('feedback_text')
  .setStyle(InputTextStyles.Paragraph)
  .setPlaceholder('Enter your feedback...')
  .setMinLength(10)
  .setMaxLength(500)
  .setRequired(true)
```

Input text styles:

| Style                       | Use it for         |
| --------------------------- | ------------------ |
| `InputTextStyles.Short`     | Single-line input. |
| `InputTextStyles.Paragraph` | Multi-line input.  |

## Components V2

Discord components v2 support richer message layouts. When you send a v2 component, include the `IS_COMPONENTS_V2` flag.

```typescript
const card = container()
  .setAccentColor(0x3366ff)
  .addComponents(
    section()
      .addComponents(textDisplay('flora runtime'))
      .setAccessory(thumbnail('https://example.com/logo.png'))
  )

await ctx.reply({
  components: [card.toJSON()],
  flags: MessageFlags.IS_COMPONENTS_V2
})
```

### Container

`container()` is the top-level wrapper for components v2.

```typescript
const card = container()
  .setAccentColor(0x3366ff)
  .setSpoiler(false)
  .addComponents(textDisplay('Build Discord bots with flora'))
```

### Section

Use `section()` to group text with an optional accessory.

```typescript
const intro = section()
  .addComponents(
    textDisplay('Welcome to flora!'),
    textDisplay('Build powerful Discord bots without running servers.')
  )
  .setAccessory(thumbnail('https://example.com/icon.png'))
```

### Text Display

```typescript
const text = textDisplay('This is display text')
```

### Thumbnail

```typescript
const image = thumbnail('https://example.com/image.png')
  .setDescription('Image description')
  .setSpoiler(true)
```

### Media Gallery

```typescript
const gallery = mediaGallery()
  .addItem('https://example.com/img1.png', { description: 'First image' })
  .addItem('https://example.com/img2.png', { spoiler: true })
  .addItem('https://example.com/img3.png')
```

### Separator

```typescript
const divider = separator(true)

const largeDivider = separator().setDivider(true).setSpacing('large')
```

### File

```typescript
const attachment = file('https://example.com/document.pdf').setSpoiler(true)
```

### Label

```typescript
const email = label('Email address')
  .setDescription('Your contact email')
  .setComponent(inputText('email'))
```

### File Upload

```typescript
const upload = fileUpload('attachment_input').setMinValues(1).setMaxValues(5).setRequired(true)
```

## Handling Interactions

Listen for `componentInteraction` to handle button clicks and select menu choices.

```typescript
on('componentInteraction', async (ctx) => {
  const customId = ctx.msg.data?.custom_id

  if (customId === 'yes') {
    await ctx.reply({ content: 'You clicked Yes!', ephemeral: true })
  } else if (customId === 'no') {
    await ctx.reply({ content: 'You clicked No!', ephemeral: true })
  }
})
```

### Select Menu Values

```typescript
on('componentInteraction', async (ctx) => {
  if (ctx.msg.data?.custom_id === 'role_select') {
    const values = ctx.msg.data?.values || []

    await ctx.reply({
      content: `You selected: ${values.join(', ')}`,
      ephemeral: true
    })
  }
})
```

## Complete Example

```typescript
const confirm = slash({
  name: 'confirm',
  description: 'Show a confirmation dialog',
  run: async (ctx) => {
    const row = actionRow().addComponents(
      button().setCustomId('confirm_yes').setLabel('Confirm').setStyle(ButtonStyle.Success),
      button().setCustomId('confirm_no').setLabel('Cancel').setStyle(ButtonStyle.Danger)
    )

    await ctx.reply({
      content: 'Are you sure you want to proceed?',
      components: [row.toJSON()]
    })
  }
})

on('componentInteraction', async (ctx) => {
  const customId = ctx.msg.data?.custom_id

  if (customId === 'confirm_yes') {
    await ctx.reply({ content: 'Action confirmed!', ephemeral: true })
  } else if (customId === 'confirm_no') {
    await ctx.reply({ content: 'Action cancelled.', ephemeral: true })
  }
})

createBot({ slashCommands: [confirm] })
```

## Available Builders

| Builder                 | What it creates                                 |
| ----------------------- | ----------------------------------------------- |
| `actionRow()`           | Container for up to 5 buttons or 1 select menu. |
| `button()`              | Interactive button.                             |
| `stringSelect(id)`      | Select menu with custom options.                |
| `userSelect(id)`        | Select menu for users.                          |
| `roleSelect(id)`        | Select menu for roles.                          |
| `channelSelect(id)`     | Select menu for channels.                       |
| `mentionableSelect(id)` | Select menu for users and roles.                |
| `inputText(id)`         | Text input field.                               |
| `container()`           | Components v2 container.                        |
| `section()`             | Components v2 content section.                  |
| `textDisplay(text)`     | Components v2 text display.                     |
| `thumbnail(url)`        | Components v2 thumbnail image.                  |
| `mediaGallery()`        | Components v2 image gallery.                    |
| `separator()`           | Components v2 divider or spacing.               |
| `file(url)`             | Components v2 file attachment.                  |
| `label(text)`           | Components v2 form label.                       |
| `fileUpload(id)`        | Components v2 file upload input.                |

## Tips

- Use descriptive custom IDs, like `delete_message_123` instead of `btn1`.
- Keep interactive messages small. Discord limits rows and components, and users scan simple layouts faster.
- Always reply to interactions, even if the response is ephemeral.
- Match button styles to the action: green for confirmation, red for destructive actions.
