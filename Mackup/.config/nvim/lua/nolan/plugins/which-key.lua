return {
	"folke/which-key.nvim",
	event = "VeryLazy",
	dependencies = {
		"echasnovski/mini.icons",
	},
	opts = {
		expand = 1,
		icons = {
			rules = {
				{ pattern = "select all", icon = "󰒆", color = "orange" },
				{ pattern = "clear search highlight", icon = "󰇾", color = "orange" },
				{ pattern = "toggle file explorer", cat = "filetype", name = "nvimtree" },
				{ pattern = "show local keymaps", cat = "filetype", name = "help" },
				{ pattern = "smart open files", icon = "󰈞", color = "orange" },
				{ pattern = "buffer", icon = "" },
				{ pattern = "sort", icon = "󰒺" },
				{ pattern = "split window vertically", icon = "", color = "orange" },
				{ pattern = "split window horizontally", icon = "", color = "orange" },
				{ pattern = "make splits equal", icon = "󰇼" },
				{ pattern = "close", icon = "󰅖" },
				{ pattern = "treesitter", icon = "󰔱" },
				{ pattern = "oil", icon = "󰙅" },
			},
		},
		spec = {
			{ "<leader>s", group = "Split", icon = { icon = "", hl = "WhichKeyIconOrange" } },
			{ "<leader>x", group = "Trouble", icon = { cat = "filetype", name = "trouble" } },
			{ "<leader>f", group = "Find", icon = "" },
			{ "<leader>b", group = "Buffer", icon = "" },
			{ "<leader>bs", "Sort", icon = "󰒺" },
		},
	},
	keys = {
		{
			"<leader>?",
			function()
				require("which-key").show({ global = false })
			end,
			desc = "Show local keymaps",
		},
	},
}
