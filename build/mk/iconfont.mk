resources/fonts/icons.%: build/data/fonts/$(MFEK_MODULE)IconFont.ufo
	$(MAKE) iconfont-impl FMT=$* UFO=$< OUT=$@

.PHONY: iconfont
ifeq ($(strip $(FONTFMT)),)
FONTFMT := otf ttf
endif
iconfont:
	$(foreach FMT,$(FONTFMT),$(MAKE) resources/fonts/icons.$(FMT);)


.PHONY: iconfont-impl
iconfont-impl:
	fontmake -u $(UFO) -o $(FMT) --output-path $(OUT)
