return {
	{
		"supermaven-inc/supermaven-nvim",
		opts = {
			ignore_filetypes = { markdown = true },
		},
	},
	{
		"saghen/blink.cmp",
		dependencies = {
			"rafamadriz/friendly-snippets",
			"Kaiser-Yang/blink-cmp-git",
		},
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
						preselect = true,
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
			sources = {
				default = { "git", "lsp", "path", "snippets" },
				providers = {
					git = {
						module = "blink-cmp-git",
						enabled = function()
							return vim.tbl_contains({ "octo", "gitcommit", "markdown" }, vim.bo.filetype)
						end,
						name = "Git",
						opts = {
							commit = {
								enable = false,
							},
						},
					},
				},
			},
		},
	},
}
