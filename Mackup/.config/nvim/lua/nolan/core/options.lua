local opt = vim.opt

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

opt.foldmethod = "expr"
opt.foldexpr = "nvim_treesitter#foldexpr()"
opt.foldenable = false
opt.foldtext = "nvim_treesitter#foldtext()"
