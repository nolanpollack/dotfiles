local opt = vim.opt

-- undercurl
vim.cmd([[let &t_Cs = "\e[4:3m"]])
vim.cmd([[let &t_Ce = "\e[4:0m"]])

-- Enable spell check
opt.spell = true
opt.spelllang = { "en_us" }

opt.relativenumber = true
opt.number = true

opt.ignorecase = true
opt.smartcase = true

opt.clipboard:append("unnamedplus")

opt.tabstop = 4
opt.shiftwidth = 4
opt.softtabstop = 4
opt.expandtab = true
opt.autoindent = true

opt.breakindent = true
opt.breakindentopt = "sbr"
opt.showbreak = "↳"
opt.linebreak = true

opt.foldmethod = "expr"
opt.foldexpr = "nvim_treesitter#foldexpr()"
opt.foldenable = false
opt.foldtext = "nvim_treesitter#foldtext()"

vim.filetype.add({
	filename = {
		["pre-commit"] = "bash",
	},
	extension = {
		["jenkins"] = "groovy",
	},
})

opt.termguicolors = true

opt.splitright = true
