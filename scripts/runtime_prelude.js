// Minimal runtime helpers exposed to user scripts.
const core = Deno.core;
globalThis.__floraHandlers = {};

globalThis.on = function on(event, handler) {
  if (!globalThis.__floraHandlers[event]) {
    globalThis.__floraHandlers[event] = [];
  }
  globalThis.__floraHandlers[event].push(handler);
};

globalThis.__floraDispatch = async function __floraDispatch(event, payload) {
  const handlers = globalThis.__floraHandlers[event] || [];
  for (const handler of handlers) {
    const context = {
      msg: payload,
      reply(message) {
        const options = normalizeReply(message, payload);
        if (options.interaction_id && options.token) {
          return core.ops.op_send_interaction_response(options);
        }
        return core.ops.op_send_message(options);
      },
      edit(message) {
        const options = normalizeEdit(message, payload);
        return core.ops.op_edit_message(options);
      },
    };
    await handler(context);
  }
};

globalThis.console = {
  log: (...args) => core.ops.op_log(args),
};

globalThis.registerSlashCommands = function registerSlashCommands(commands) {
  if (!globalThis.__floraGuildId) return;
  return core.ops.op_upsert_guild_commands({
    guildId: globalThis.__floraGuildId,
    commands,
  });
};

function normalizeReply(message, payload) {
  if (payload?.interaction_token) {
    return normalizeInteractionReply(message, payload);
  }

  const base = { channel_id: payload.channel_id };

  if (typeof message === "string") {
    return { ...base, message_id: payload.id, content: message };
  }

  if (message && typeof message === "object") {
    const normalized = { ...base, ...message };
    const explicitReplyTo = message.replyTo ?? message.reply_to;

    if (explicitReplyTo === null) {
      delete normalized.message_id;
    } else if (explicitReplyTo !== undefined) {
      normalized.message_id = explicitReplyTo;
    } else if (payload?.id) {
      normalized.message_id = payload.id;
    }

    delete normalized.replyTo;
    delete normalized.reply_to;
    return normalized;
  }

  return { ...base, message_id: payload.id, content: String(message) };
}

function normalizeEdit(message, payload) {
  if (!payload?.id || !payload?.channel_id) {
    throw new Error("Message edit requires a message payload");
  }

  const base = { channel_id: payload.channel_id, message_id: payload.id };

  if (typeof message === "string") {
    return { ...base, content: message };
  }

  if (message && typeof message === "object") {
    return { ...base, ...message };
  }

  return { ...base, content: String(message) };
}

function normalizeInteractionReply(message, payload) {
  const base = {
    interaction_id: payload.interaction_id ?? payload.id,
    token: payload.interaction_token,
  };

  if (typeof message === "string") {
    return { ...base, content: message };
  }

  if (message && typeof message === "object") {
    const normalized = { ...base, ...message };
    if (message.ephemeral !== undefined) {
      normalized.ephemeral = message.ephemeral;
    }
    return normalized;
  }

  return { ...base, content: String(message) };
}
