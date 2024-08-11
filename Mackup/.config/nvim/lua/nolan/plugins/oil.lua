return {
	"stevearc/oil.nvim",
	opts = {},
	dependencies = { { "echasnovski/mini.icons", opts = {} } },
	config = function()
		require("oil").setup({
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
		})
		vim.keymap.set("n", "<leader>o", "<cmd>Oil --float<CR>", { desc = "Open oil" })
	end,
}
