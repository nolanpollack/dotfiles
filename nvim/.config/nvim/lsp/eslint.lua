vim.lsp.config("eslint", {
	root_dir = function(bufnr, on_dir)
		local root = vim.fs.root(bufnr, { "eslint.config.ts", "eslint.config.js", ".git" })
		if root then
			on_dir(root)
		end
	end,
	settings = {
		experimental = {
			useFlatConfig = true,
		},
		format = true,
		workingDirectory = { mode = "location" },
	},
})
