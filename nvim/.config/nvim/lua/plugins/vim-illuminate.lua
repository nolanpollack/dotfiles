return {
    -- TODO: This seems to break all the time. Is there a better solution?
	"RRethy/vim-illuminate",
	config = function()
		require("illuminate").configure({
			under_cursor = false,
		})
	end,
}
