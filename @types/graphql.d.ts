
declare module '*/customer.graphql' {
  import { DocumentNode } from 'graphql';
  const defaultDocument: DocumentNode;
  export const CreateCustomer: DocumentNode;
export const CreateCustomerWallet: DocumentNode;
export const GetCustomerWallet: DocumentNode;
export const GetCustomerTreasury: DocumentNode;
export const GetCustomerCollectibles: DocumentNode;

  export default defaultDocument;
}
    

declare module '*/drop.graphql' {
  import { DocumentNode } from 'graphql';
  const defaultDocument: DocumentNode;
  export const MintNft: DocumentNode;
export const GetDrop: DocumentNode;
export const GetDrops: DocumentNode;

  export default defaultDocument;
}
    

declare module '*/mint.graphql' {
  import { DocumentNode } from 'graphql';
  const defaultDocument: DocumentNode;
  export const MintDrop: DocumentNode;

  export default defaultDocument;
}
    

declare module '*/subscription.graphql' {
  import { DocumentNode } from 'graphql';
  const defaultDocument: DocumentNode;
  export const Subscribe: DocumentNode;
export const Unsubscribe: DocumentNode;
export const GetSubscription: DocumentNode;

  export default defaultDocument;
}
    

declare module '*/collectibles.graphql' {
  import { DocumentNode } from 'graphql';
  const defaultDocument: DocumentNode;
  export const GetCollectibles: DocumentNode;

  export default defaultDocument;
}
    

declare module '*/me.graphql' {
  import { DocumentNode } from 'graphql';
  const defaultDocument: DocumentNode;
  export const GetMe: DocumentNode;

  export default defaultDocument;
}
    

declare module '*/project.graphql' {
  import { DocumentNode } from 'graphql';
  const defaultDocument: DocumentNode;
  export const GetProjectDrop: DocumentNode;
export const GetProjectDrops: DocumentNode;

  export default defaultDocument;
}
    