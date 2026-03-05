const TAG_ALLOWED_ROLE = '880150867792232609'

export default slash({
  name: 'tag',
  description: 'Manage server tags',
  subcommands: [
    {
      name: 'create',
      description: 'Create a new tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        },
        {
          name: 'content',
          description: 'The tag content',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: 'You do not have permission to create tags.',
            ephemeral: true
          })
        }

        const name = ctx.options.name as string
        const content = ctx.options.content as string

        const tagStore = kv.store('tags')
        const existing = await tagStore.get(name)

        if (existing) {
          return await ctx.reply({
            content: `A tag with the name "**${name}**" already exists.`,
            ephemeral: true
          })
        }

        await tagStore.set(name, content)
        return await ctx.reply(`Created tag "**${name}**"!`)
      }
    },
    {
      name: 'view',
      description: 'View a tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        const name = ctx.options.name as string

        const tagStore = kv.store('tags')
        const content = await tagStore.get(name)

        if (!content) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true
          })
        }

        return await ctx.reply(content)
      }
    },
    {
      name: 'edit',
      description: 'Edit an existing tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        },
        {
          name: 'content',
          description: 'The new tag content',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: 'You do not have permission to edit tags.',
            ephemeral: true
          })
        }

        const name = ctx.options.name as string
        const content = ctx.options.content as string

        const tagStore = kv.store('tags')
        const existing = await tagStore.get(name)

        if (!existing) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true
          })
        }

        await tagStore.set(name, content)
        return await ctx.reply(`Updated tag "**${name}**"!`)
      }
    },
    {
      name: 'delete',
      description: 'Delete a tag',
      options: [
        {
          name: 'name',
          description: 'The tag name',
          type: 'string',
          required: true
        }
      ],
      async run(ctx) {
        if (!hasRole(ctx, TAG_ALLOWED_ROLE)) {
          return await ctx.reply({
            content: 'You do not have permission to delete tags.',
            ephemeral: true
          })
        }

        const name = ctx.options.name as string

        const tagStore = kv.store('tags')
        const existing = await tagStore.get(name)

        if (!existing) {
          return await ctx.reply({
            content: `Tag "**${name}**" not found.`,
            ephemeral: true
          })
        }

        await tagStore.delete(name)
        return await ctx.reply(`Deleted tag "**${name}**"!`)
      }
    }
  ]
})
