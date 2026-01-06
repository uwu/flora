var flora = (function(exports) {


//#region src/kv.ts
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
				expiration: options?.expiration ?? null,
				metadata: options?.metadata ?? null
			});
		}
		/**
		* Update just the metadata for a key without changing the value.
		*
		* @param key - The key to update
		* @param metadata - The metadata to set, or null to remove metadata
		*/
		async updateMetadata(key, metadata) {
			await Deno.core.ops.op_kv_update_metadata(this.#storeName, key, metadata ?? null);
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
				prefix: options?.prefix ?? null,
				limit: options?.limit ?? null,
				cursor: options?.cursor ?? null
			}, this.#storeName);
		}
	};
	function store(name) {
		return new KvStore(name);
	}
	const kv = { store };

//#endregion
//#region src/index.ts
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
	function defineCommand(command) {
		return command;
	}
	function defineSlashCommand(command) {
		return command;
	}
	function createBot(options) {
		const prefix = options.prefix ?? "!";
		const commands = options.commands ?? options.prefixCommands ?? [];
		const slashCommands = options.slashCommands ?? [];
		on("messageCreate", async (ctx) => {
			if (!ctx.msg || !ctx.msg.content) return;
			if (ctx.msg.author?.bot) return;
			const content = ctx.msg.content.trim();
			if (!content.startsWith(prefix)) return;
			const body = content.slice(prefix.length).trim();
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
			const command = slashCommands.find((cmd) => cmd.name === ctx.msg.command_name);
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
		globalThis.__floraSubcommands = globalThis.__floraSubcommands || {};
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
exports.EmbedBuilder = EmbedBuilder;
exports.createBot = createBot;
exports.defineCommand = defineCommand;
exports.defineSlashCommand = defineSlashCommand;
exports.embed = embed;
exports.getSubcommand = getSubcommand;
exports.getSubcommandGroup = getSubcommandGroup;
exports.hasRole = hasRole;
exports.kv = kv;
return exports;
})({});

;(function (global) {
  if (!global.flora) return;
  global.createBot = global.flora.createBot;
  global.defineCommand = global.flora.defineCommand;
  global.defineSlashCommand = global.flora.defineSlashCommand;
  global.kv = global.flora.kv;
})(globalThis);
