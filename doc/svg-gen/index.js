const {createSVGWindow} = require('svgdom')
const window = createSVGWindow()
const document = window.document
const {SVG, registerWindow} = require('@svgdotjs/svg.js')
const fs = require('fs')
const path = require("path")
const colors = require('material-colors')
require('@svgdotjs/svg.topath.js')

registerWindow(window, document)

const onionLayers = generateOnionLayersDiagram();
fs.writeFileSync('doc/images/onion-layers.svg', onionLayers.svg())

function generateOnionLayersDiagram() {
    const width = 410;
    const height = 410;
    const draw = SVG(document.documentElement).size(width, height);
    // const css = fs.readFileSync(path.resolve(__dirname, "styles.css"));
    // draw.element('style').words(css)
    draw.style("@import 'styles.css';");
    const defaultFontSize = 15;
    const defaultTextColor = colors.grey[900];
    const defaultArrowColor = colors.green[900];
    const defaultArrowWidth = 2;
    const defaultArrowTextColor = colors.grey[700];
    const baseFont = {
        family: 'sans-serif',
        fill: defaultTextColor,
        size: defaultFontSize,
    }
    const arrowPattern = createArrowPattern();
    const defaultArrowHead = arrowHead();

    // Layers
    const layerColorDepth = 100;
    const infrastructureLayerColor = colors.yellow[layerColorDepth];
    const managementLayerColor = colors.lightGreen[layerColorDepth];
    const processingLayerColor = colors.lightBlue[layerColorDepth];
    const baseLayerColor = colors.blueGrey[layerColorDepth];
    const infrastructureLayer = layer(4, 'infrastructure', infrastructureLayerColor, [
        'GUI',
        'API',
        'Persistence',
        'Server',
    ]);
    layer(3, 'management', managementLayerColor);
    layer(2, 'processing', processingLayerColor);
    const baseLayer = layer(1, 'base', baseLayerColor);

    // Arrows
    drawArrow(infrastructureLayer.x(), infrastructureLayer.cy(), baseLayer.cx(), baseLayer.cy(), {
        text: 'may use code in',
        patternOrColor: arrowPattern,
        width: 10,
        drawHead: false,
        pathClass: 'go-right',
        textClass: 'arrow-label',
        useClipping: true,
    });

    function arrowHead() {
        return draw.marker(10, 7, (add) => {
            add.polygon('0,0 10,3.5 0,7').fill(defaultArrowColor);
        });
    }

    function arcPath(radius, cx, cy, sweep) {
        return [
            // Start at 9
            ['M', cx - radius, cy],
            // Go to 3
            ['A', -radius, -radius, 0, 0, sweep, cx + radius, cy],
        ];
    }

    function layer(index, label, color, components = []) {
        const g = draw.group();
        const spacing = 50;
        const radius = index * spacing;
        const circle = g
            .circle(radius * 2)
            .stroke({color: colors.grey[500], width: 2})
            .center(width / 2, height / 2)
            .fill(color)
            .attr('fill-opacity', 1.0);
        const pathRadius = radius - spacing / 2;
        const radiusFix = defaultFontSize / 3;
        const upperArc = arcPath(
            pathRadius - radiusFix,
            circle.cx(),
            circle.cy(),
            1
        );
        g.textPath(label, upperArc)
            .attr('text-anchor', 'middle')
            .attr('startOffset', '50%')
            .attr('letter-spacing', 1)
            .font(baseFont);
        const lowerArc = arcPath(
            pathRadius + radiusFix,
            circle.cx(),
            circle.cy(),
            0
        );
        for (let i = 0; i < components.length; i++) {
            const segmentLength = (1 / components.length) * 100;
            const offset = i * segmentLength + segmentLength / 2;
            g.textPath(components[i], lowerArc)
                .attr('text-anchor', 'middle')
                .attr('startOffset', `${offset}%`)
                .attr('letter-spacing', 1)
                .font({...baseFont, fill: colors.grey[500]});
        }
        return circle;
    }

    function createArrowPattern() {
        return draw.pattern(30, 20, (add) => {
            const g = add.group();
            g.line(0, 3.5, 10, 3.5).stroke({color: defaultArrowColor, width: 1});
            g.polygon('10,0 20,3.5 10,7').fill(defaultArrowColor);
        });
    }

    function drawArrow(x1, y1, x2, y2, {
        patternOrColor = defaultArrowColor,
        width = defaultArrowWidth,
        head = defaultArrowHead,
        text,
        textColor = defaultArrowTextColor,
        drawHead = true,
        pathClass = undefined,
        textClass = undefined,
        useClipping = false,
    }) {
        // Group for clipping
        const g = draw.group();
        // Arrow itself
        const line = useClipping ? g.line(x1 - 100, y1, x2 + 100, y2) : g.line(x1, y1, x2, y2);
        const path = line.toPath().addClass(pathClass);
        path.stroke({color: patternOrColor, width});
        // Head
        if (drawHead) {
            path.marker('end', head.size(8, 8))
        }
        // Text
        draw.textPath()
            .plot(path.array())
            .text(add => {
                add.tspan(text).dy(-10)
            })
            .addClass(textClass)
            .font({...baseFont, anchor: 'middle', startOffset: '50%', fill: textColor});
        // Clip (useful for keeping CSS transform animation within bounds)
        if (useClipping) {
            const clip = g.clip()
                .add(g.polygon().plot([[x1, y1 - width], [x2, y2 - width], [x2, y1 + width], [12, y2 + width]]));
            g.clipWith(clip);
        }
    }

    return draw;
}