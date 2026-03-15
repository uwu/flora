import { describe, expect, it } from 'vite-plus/test'

import { embed } from './embed'

describe('EmbedBuilder', () => {
  it('builds embed with fluent setters', () => {
    const built = embed()
      .setTitle('Hello')
      .setDescription('World')
      .setUrl('https://example.com')
      .setColor(0xff00ff)
      .setFooter('footer', 'https://img/footer.png')
      .setImage('https://img/image.png')
      .setThumbnail('https://img/thumb.png')
      .setAuthor('author', { url: 'https://auth', iconUrl: 'https://img/auth.png' })
      .addField('Field 1', 'Value 1')
      .addField('Field 2', 'Value 2', true)
      .toJSON()

    expect(built).toEqual({
      title: 'Hello',
      description: 'World',
      url: 'https://example.com',
      color: 0xff00ff,
      footer: { text: 'footer', iconUrl: 'https://img/footer.png' },
      image: { url: 'https://img/image.png' },
      thumbnail: { url: 'https://img/thumb.png' },
      author: { name: 'author', url: 'https://auth', iconUrl: 'https://img/auth.png' },
      fields: [
        { name: 'Field 1', value: 'Value 1', inline: false },
        { name: 'Field 2', value: 'Value 2', inline: true }
      ]
    })
  })

  it('merges initial state', () => {
    const built = embed({ title: 'initial' }).setDescription('desc').toJSON()
    expect(built.title).toBe('initial')
    expect(built.description).toBe('desc')
  })
})
