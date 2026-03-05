return {
	"windwp/nvim-ts-autotag",
	cond = not vim.g.scrollback_mode,
	event = { "BufReadPre", "BufNewFile" },
	opts = {},
}
