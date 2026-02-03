return {
	-- TODO: Is there any way to delete surrounding function? Turn test(inner) into inner
	"kylechui/nvim-surround",
	event = { "BufReadPre", "BufNewFile" },
	version = "*",
	config = true,
}
