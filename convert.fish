mkdir output
for file in *.png
  convert $file -resize 333x349\> -gravity center -background none -extent 365x381 \
  \( +clone -background black -shadow 80x8+0+0 \) +swap -background none -layers merge +repage ./output/$file
end
