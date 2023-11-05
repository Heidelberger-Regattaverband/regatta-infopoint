import Table from "sap/m/Table";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import JSONModel from "sap/ui/model/json/JSONModel";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import Button, { Button$PressEvent } from "sap/m/Button";
import MessageToast from "sap/m/MessageToast";
import Context from "sap/ui/model/Context";
import { SearchField$SearchEvent } from "sap/m/SearchField";
import { ListBase$SelectEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ParticipatingClubsTable extends BaseController {

  private table: Table;
  private participatingClubsModel: JSONModel;

  async onInit(): Promise<void> {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());

    this.table = super.getView()?.byId("clubsTable") as Table;

    this.participatingClubsModel = await super.createJSONModel(`/api/regattas/${this.getRegattaId()}/participating_clubs`, this.table);
    super.setViewModel(this.participatingClubsModel, "clubs");

    super.getRouter()?.getRoute("participatingClubs")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadModel(), this);
  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  onFilterSearch(event: SearchField$SearchEvent): void {
    const searchFilters: Filter[] = [];
    const query: string | undefined = event.getParameters().query?.trim();
    if (query) {
      searchFilters.push(
        new Filter({
          filters: [
            new Filter("shortName", FilterOperator.Contains, query),
            new Filter("longName", FilterOperator.Contains, query),
            new Filter("abbreviation", FilterOperator.Contains, query),
            new Filter("city", FilterOperator.Contains, query)
          ],
          and: false
        }))
    }
    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding.filter(searchFilters);
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    await this.loadModel();
    MessageToast.show(this.i18n("msg.dataUpdated"));
    source.setEnabled(true);
  }

  onItemPress(event: ListBase$SelectEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameters().listItem;
    if (selectedItem) {
      const bindingCtx: Context | undefined | null = selectedItem.getBindingContext("clubs");
      const club: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.getRouter().navTo("clubParticipations", { clubId: club.id }, false /* history*/);
    }
  }

  private async loadModel(): Promise<void> {
    await super.updateJSONModel(this.participatingClubsModel, `/api/regattas/${this.getRegattaId()}/participating_clubs`, this.table)
  }
}