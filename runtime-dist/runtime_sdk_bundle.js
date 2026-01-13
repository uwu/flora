var flora = (function(exports) {


//#region src/sdk/commands.ts
function prefix(command) {
	return command;
}
function slash(command) {
	return command;
}
function createBot(options) {
	const prefix$1 = options.prefix ?? "!";
	const commands = options.commands ?? options.prefixCommands ?? [];
	const slashCommands = options.slashCommands ?? [];
	on("messageCreate", async (ctx) => {
		if (!ctx.msg || !ctx.msg.content) return;
		if (ctx.msg.author?.bot) return;
		const content = ctx.msg.content.trim();
		if (!content.startsWith(prefix$1)) return;
		const body = content.slice(prefix$1.length).trim();
		const [commandName, ...args] = body.split(/\s+/);
		const command = commands.find((cmd) => cmd.name === commandName);
		if (!command) return;
		await command.run({
			...ctx,
			args
		});
	});
	on("interactionCreate", async (ctx) => {
		if (!ctx.msg) return;
		const command = slashCommands.find((cmd) => cmd.name === ctx.msg.commandName);
		if (!command) return;
		if (command.subcommands && command.subcommands.length > 0) {
			await handleSubcommand(ctx, command);
		} else if (command.run) {
			const rawData = ctx.msg.data;
			const options$1 = flattenInteractionOptions(rawData?.options || []);
			await command.run({
				...ctx,
				options: options$1
			});
		}
	});
	if (slashCommands.length && typeof registerSlashCommands === "function") {
		const flattenedCommands = flattenCommands(slashCommands);
		registerSlashCommands(flattenedCommands);
	}
}
function flattenCommands(commands) {
	const subcommands = globalThis.__floraSubcommands;
	globalThis.__floraSubcommands = subcommands || {};
	return commands.map((cmd) => {
		if (cmd.subcommands && cmd.subcommands.length > 0) {
			const submap = {};
			cmd.subcommands.forEach((sub) => {
				submap[sub.name] = sub.run;
			});
			globalThis.__floraSubcommands[cmd.name] = submap;
			return {
				name: cmd.name,
				description: cmd.description,
				options: cmd.subcommands.map((sub) => ({
					name: sub.name,
					description: sub.description,
					type: "subcommand",
					options: sub.options
				}))
			};
		}
		return {
			name: cmd.name,
			description: cmd.description,
			options: cmd.options
		};
	});
}
async function handleSubcommand(ctx, command) {
	const rawData = ctx.msg.data;
	if (!rawData?.options || !Array.isArray(rawData.options)) {
		return;
	}
	const firstOption = rawData.options[0];
	if (!firstOption) return;
	const subcommandName = firstOption.name;
	const subcommandMap = globalThis.__floraSubcommands?.[command.name];
	if (!subcommandMap) return;
	const subcommandHandler = subcommandMap[subcommandName];
	if (!subcommandHandler) return;
	const subcommandOptions = firstOption.options || [];
	const flatOptions = flattenInteractionOptions(subcommandOptions);
	const enrichedCtx = {
		...ctx,
		options: flatOptions
	};
	await subcommandHandler(enrichedCtx);
}
function flattenInteractionOptions(options) {
	const result = {};
	for (const opt of options) {
		if (opt.type === 1 || opt.type === 2) {
			Object.assign(result, flattenInteractionOptions(opt.options || []));
		} else {
			result[opt.name] = opt.value;
		}
	}
	return result;
}

//#endregion
//#region src/sdk/embed.ts
var EmbedBuilder = class {
	#embed;
	constructor(initial = {}) {
		this.#embed = { ...initial };
	}
	setTitle(title) {
		this.#embed.title = title;
		return this;
	}
	setDescription(description) {
		this.#embed.description = description;
		return this;
	}
	setUrl(url) {
		this.#embed.url = url;
		return this;
	}
	setColor(color) {
		this.#embed.color = color;
		return this;
	}
	setTimestamp(timestamp) {
		this.#embed.timestamp = timestamp;
		return this;
	}
	setFooter(text, iconUrl) {
		this.#embed.footer = {
			text,
			iconUrl
		};
		return this;
	}
	setImage(url) {
		this.#embed.image = { url };
		return this;
	}
	setThumbnail(url) {
		this.#embed.thumbnail = { url };
		return this;
	}
	setAuthor(name, options) {
		this.#embed.author = {
			name,
			...options
		};
		return this;
	}
	addField(name, value, inline = false) {
		const field = {
			name,
			value,
			inline
		};
		this.#embed.fields = [...this.#embed.fields ?? [], field];
		return this;
	}
	addFields(fields) {
		this.#embed.fields = [...this.#embed.fields ?? [], ...fields];
		return this;
	}
	setFields(fields) {
		this.#embed.fields = [...fields];
		return this;
	}
	toJSON() {
		return { ...this.#embed };
	}
};
function embed(initial) {
	return new EmbedBuilder(initial);
}

//#endregion
//#region src/sdk/helpers.ts
function hasRole(ctx, roleId) {
	return ctx.msg.member?.roles?.includes(roleId) ?? false;
}
function getSubcommand(ctx) {
	const rawData = ctx.msg.data;
	if (!rawData?.options || !Array.isArray(rawData.options)) return undefined;
	return rawData.options[0]?.name;
}
function getSubcommandGroup(ctx) {
	const rawData = ctx.msg.data;
	if (!rawData?.options || !Array.isArray(rawData.options)) return undefined;
	const firstOption = rawData.options[0];
	if (!firstOption) return undefined;
	const type = firstOption.type;
	if (type === 2) {
		return firstOption.name;
	}
	return undefined;
}

//#endregion
//#region src/sdk/kv.ts
var KvStore = class {
	#storeName;
	constructor(storeName) {
		this.#storeName = storeName;
	}
	/**
	* Get a value from the store.
	*
	* @param key - The key to retrieve
	* @returns The value, or null if not found
	*/
	async get(key) {
		return await Deno.core.ops.op_kv_get(this.#storeName, key);
	}
	/**
	* Get a value from the store along with its metadata.
	*
	* @param key - The key to retrieve
	* @returns Object with value and optional metadata
	*/
	async getWithMetadata(key) {
		const result = await Deno.core.ops.op_kv_get_with_metadata(this.#storeName, key);
		if (result === null) {
			return { value: null };
		}
		const [value, metadata] = result;
		return {
			value,
			metadata: metadata?.metadata
		};
	}
	/**
	* Set a value in the store.
	*
	* The value size is limited to 1MB.
	*
	* @param key - The key to set
	* @param value - The value to store (max 1MB)
	* @param options - Optional expiration (Unix timestamp) and metadata
	*/
	async set(key, value, options) {
		await Deno.core.ops.op_kv_set(this.#storeName, key, value, {
			expiration: options?.expiration ?? undefined,
			metadata: options?.metadata ?? undefined
		});
	}
	/**
	* Update just the metadata for a key without changing the value.
	*
	* @param key - The key to update
	* @param metadata - The metadata to set, or null to remove metadata
	*/
	async updateMetadata(key, metadata) {
		await Deno.core.ops.op_kv_update_metadata(this.#storeName, key, metadata ?? undefined);
	}
	/**
	* Delete a key from the store.
	*
	* @param key - The key to delete
	*/
	async delete(key) {
		await Deno.core.ops.op_kv_delete(this.#storeName, key);
	}
	/**
	* List all keys in the store with cursor-based pagination.
	*
	* @param options - Optional prefix filter, limit (default 100, max 1000), and cursor for pagination
	* @returns Paginated result with keys, list_complete flag, and cursor for next page
	*/
	async list(options) {
		return await Deno.core.ops.op_kv_list_keys({
			prefix: options?.prefix ?? undefined,
			limit: options?.limit ?? undefined,
			cursor: options?.cursor ?? undefined
		}, this.#storeName);
	}
};
function store(name) {
	return new KvStore(name);
}
const kv = { store };

//#endregion
//#region src/sdk/rest.ts
const ops = Deno.core.ops;
const rest = {
	sendMessage: (args) => ops.op_send_message(args),
	editMessage: (args) => ops.op_edit_message(args),
	deleteMessage: (args) => ops.op_delete_message(args),
	bulkDeleteMessages: (args) => ops.op_bulk_delete_messages(args),
	pinMessage: (args) => ops.op_pin_message(args),
	unpinMessage: (args) => ops.op_unpin_message(args),
	crosspostMessage: (args) => ops.op_crosspost_message(args),
	fetchMessage: (args) => ops.op_fetch_message(args),
	fetchMessages: (args) => ops.op_fetch_messages(args),
	addReaction: (args) => ops.op_add_reaction(args),
	removeReaction: (args) => ops.op_remove_reaction(args),
	clearReactions: (args) => ops.op_clear_reactions(args),
	sendInteractionResponse: (args) => ops.op_send_interaction_response(args),
	deferInteractionResponse: (args) => ops.op_defer_interaction_response(args),
	updateInteractionResponse: (args) => ops.op_update_interaction_response(args),
	editOriginalInteractionResponse: (args) => ops.op_edit_original_interaction_response(args),
	deleteOriginalInteractionResponse: (args) => ops.op_delete_original_interaction_response(args),
	createFollowupMessage: (args) => ops.op_create_followup_message(args),
	editFollowupMessage: (args) => ops.op_edit_followup_message(args),
	deleteFollowupMessage: (args) => ops.op_delete_followup_message(args),
	upsertGuildCommands: (args) => ops.op_upsert_guild_commands(args),
	createGuildCommand: (args) => ops.op_create_guild_command(args),
	editGuildCommand: (args) => ops.op_edit_guild_command(args),
	deleteGuildCommand: (args) => ops.op_delete_guild_command(args),
	getGuildCommands: (args) => ops.op_get_guild_commands(args),
	getGuildCommand: (args) => ops.op_get_guild_command(args),
	editGuildCommandPermissions: (args) => ops.op_edit_guild_command_permissions(args),
	getGuildCommandsPermissions: (args) => ops.op_get_guild_commands_permissions(args),
	getGuildCommandPermissions: (args) => ops.op_get_guild_command_permissions(args),
	kickMember: (args) => ops.op_kick_member(args),
	banMember: (args) => ops.op_ban_member(args),
	unbanMember: (args) => ops.op_unban_member(args),
	addMemberRole: (args) => ops.op_add_member_role(args),
	removeMemberRole: (args) => ops.op_remove_member_role(args),
	editMember: (args) => ops.op_edit_member(args),
	createChannel: (args) => ops.op_create_channel(args),
	editChannel: (args) => ops.op_edit_channel(args),
	deleteChannel: (args) => ops.op_delete_channel(args),
	createThread: (args) => ops.op_create_thread(args),
	createThreadFromMessage: (args) => ops.op_create_thread_from_message(args),
	joinThread: (args) => ops.op_join_thread(args),
	leaveThread: (args) => ops.op_leave_thread(args),
	addThreadMember: (args) => ops.op_add_thread_member(args),
	removeThreadMember: (args) => ops.op_remove_thread_member(args),
	executeWebhook: (args) => ops.op_execute_webhook(args),
	editWebhook: (args) => ops.op_edit_webhook(args),
	deleteWebhook: (args) => ops.op_delete_webhook(args)
};

//#endregion
exports.EmbedBuilder = EmbedBuilder;
exports.KvStore = KvStore;
exports.createBot = createBot;
exports.embed = embed;
exports.flattenCommands = flattenCommands;
exports.flattenInteractionOptions = flattenInteractionOptions;
exports.getSubcommand = getSubcommand;
exports.getSubcommandGroup = getSubcommandGroup;
exports.handleSubcommand = handleSubcommand;
exports.hasRole = hasRole;
exports.kv = kv;
exports.prefix = prefix;
exports.rest = rest;
exports.slash = slash;
exports.store = store;
return exports;
})({});

;(function (global) {
  if (!global.flora) return;
  global.createBot = global.flora.createBot;
  global.prefix = global.flora.prefix;
  global.slash = global.flora.slash;
  global.hasRole = global.flora.hasRole;
  global.getSubcommand = global.flora.getSubcommand;
  global.getSubcommandGroup = global.flora.getSubcommandGroup;
  global.kv = global.flora.kv;
  global.EmbedBuilder = global.flora.EmbedBuilder;
  global.embed = global.flora.embed;
})(globalThis);
