return {
	"jmbuhr/otter.nvim",
	cond = not vim.g.scrollback_mode,
	dependencies = {
		"nvim-treesitter/nvim-treesitter",
	},
	opts = {},
}
