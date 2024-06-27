export function render(): {
  (svg: any, g: any): void;
  createNodes(value: any, ...args: any[]): (selection: any, g: any, shapes: any) => any;
  createClusters(value: any, ...args: any[]): (selection: any, g: any) => any;
  createEdgeLabels(value: any, ...args: any[]): (selection: any, g: any) => any;
  createEdgePaths(value: any, ...args: any[]): (selection: any, g: any, arrows: any) => any;
  shapes(
    value: any,
    ...args: any[]
  ):
    | {
        rect: (parent: any, bbox: any, node: any) => any;
        ellipse: (parent: any, bbox: any, node: any) => any;
        circle: (parent: any, bbox: any, node: any) => any;
        diamond: (parent: any, bbox: any, node: any) => any;
      }
    | any;
  arrows(
    value: any,
    ...args: any[]
  ):
    | {
        normal: (parent: any, id: any, edge: any, type: any) => void;
        vee: (parent: any, id: any, edge: any, type: any) => void;
        undirected: (parent: any, id: any, edge: any, type: any) => void;
      }
    | any;
};
