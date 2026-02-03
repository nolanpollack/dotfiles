return {
	{
		-- TODO: I have indent-blankline already. Does both do anything?
		"echasnovski/mini.indentscope",
		version = false,
		event = { "BufReadPre", "BufNewFile" },
		opts = {
			symbol = "│",
		},
	},
	{
		"echasnovski/mini.ai",
		version = false,
		opts = {},
	},
}
