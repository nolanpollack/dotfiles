return {
	"echasnovski/mini.ai",
	event = { "BufReadPre", "BufNewFile" },
	config = function()
		require("mini.ai").setup()
	end,
}
