return {
	"saghen/blink.cmp",
	version = "1.8.0",
	dependencies = {
		"rafamadriz/friendly-snippets", -- useful snippets
	}, -- @type blink.cmp.Config
	opts = {
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
}
