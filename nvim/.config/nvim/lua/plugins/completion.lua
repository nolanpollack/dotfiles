return {
	{
		"supermaven-inc/supermaven-nvim",
		opts = {},
	},
	{
		"saghen/blink.cmp",
		version = "1.8.0",
		-- @type blink.cmp.Config
		opts = {
			completion = {
				menu = {
					border = "rounded",
					draw = {
						columns = {
							{ "kind_icon" },
							{ "label", "label_description", gap = 1 },
							{ "kind" },
						},
						treesitter = { "lsp" },
					},
					direction_priority = { "n", "s" },
				},
				list = {
					selection = {
						preselect = false,
						auto_insert = false,
					},
				},
				documentation = {
					auto_show = true,
					auto_show_delay_ms = 500,
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
