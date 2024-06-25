return {
	"folke/which-key.nvim",
	event = "VeryLazy",
	init = function()
		vim.o.timeout = true
		vim.o.timeoutlen = 500
		vim.keymap.set("n", "?", ":WhichKey<CR>", { desc = "WhichKey" })
	end,
	opts = {},
}
