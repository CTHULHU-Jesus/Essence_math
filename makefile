.PHONY: all
all: index.scss bundle.js index.html


index.html: web/index.html
	cp $< $@

index.scss: web/index.scss
	cp $< $@

bundle.js: web/bundle.js
	$(MAKE) -C web bundle.js
	cp $< $@
