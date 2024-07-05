return {
	"MagicDuck/grug-far.nvim",
	config = function()
		require("grug-far").setup({})
		vim.keymap.set(
			"n",
			"<leader>fr",
			"<cmd>lua require('grug-far').grug_far({ prefills = { flags = vim.fn.expand('%') } })<CR>",
			{ desc = "Find and Replace" }
		)
	end,
}
