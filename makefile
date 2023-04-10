.PHONY: all
all: index.css bundle.js index.html


index.html: web/index.html
	cp $< $@

index.css: web/index.css
	$(MAKE) -C web index.css
	cp $< $@

bundle.js: web/bundle.js
	$(MAKE) -C web bundle.js
	cp $< $@
