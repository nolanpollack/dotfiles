return {
	"pwntester/octo.nvim",
    cmd = "Octo",
	dependencies = {
		{ "nvim-lua/plenary.nvim" },
		{ "nvim-telescope/telescope.nvim" },
		{ "echasnovski/mini.icons" },
	},
	opts = { suppress_missing_scope = { projects_v2 = true } },
}
