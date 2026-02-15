import { createConsola } from 'consola/basic'

export const logger = createConsola({
  formatOptions: {
    colors: true,
    date: false,
    compact: true
  }
})
