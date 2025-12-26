return {
	{ "supermaven-inc/supermaven-nvim" },
	{
		"saghen/blink.cmp",
		version = "1.8.0",
		dependencies = {
			"saghen/blink.compat",
			"supermaven-inc/supermaven-nvim",
		},
		-- @type blink.cmp.Config
		opts = {
			sources = {
				providers = {
					supermaven = {
						name = "supermaven",
						module = "blink.compat.source",
					},
				},
			},
			completion = {
				menu = {
					border = "rounded",

					draw = {
						columns = {
							{ "kind_icon" },
							{ "label", "label_description", gap = 1 },
							{ "kind" }, -- Add this to show the completion type name
						},
					},
					-- Make sure menu doesn't overlap with ghost text (copilot)
					direction_priority = function()
						local ctx = require("blink.cmp").get_context()
						local item = require("blink.cmp").get_selected_item()
						if ctx == nil or item == nil then
							return { "s", "n" }
						end

						local item_text = item.textEdit ~= nil and item.textEdit.newText
							or item.insertText
							or item.label
						local is_multi_line = item_text:find("\n") ~= nil

						-- after showing the menu upwards, we want to maintain that direction
						-- until we re-open the menu, so store the context id in a global variable
						if is_multi_line or vim.g.blink_cmp_upwards_ctx_id == ctx.id then
							vim.g.blink_cmp_upwards_ctx_id = ctx.id
							return { "n", "s" }
						end
						return { "s", "n" }
					end,
				},
				list = {
					selection = {
						preselect = false,
						auto_insert = false,
					},
				},
			},
			signature = { enabled = true },
			keymap = {
				preset = "default",
				["<C-k>"] = { "select_prev", "fallback" },
				["<C-j>"] = { "select_next", "fallback" },
				["<CR>"] = { "accept", "fallback" },
			},
			cmdline = {
				keymap = { preset = "inherit" },
				completion = {
					menu = { auto_show = true },
					list = {
						selection = {
							preselect = false,
							auto_insert = false,
						},
					},
				},
			},
		},
	},
}
