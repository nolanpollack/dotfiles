return {
	"stevearc/oil.nvim",
	dependencies = { { "echasnovski/mini.icons", opts = {} } },
	keys = {
		{ "<leader>o", "<cmd>Oil --float<CR>", desc = "Open oil" },
	},
	opts = {
		keymaps = {
			["<leader>sv"] = {
				"actions.select",
				opts = { vertical = true },
				desc = "Open the entry in a vertical split",
			},
			["<leader>sh"] = {
				"actions.select",
				opts = { horizontal = true },
				desc = "Open the entry in a horizontal split",
			},
		},
	},
}
